"""Upload screenshots to S3-compatible storage (Cloudflare R2)."""

from __future__ import annotations

import base64
import logging

import aioboto3

from server.config import settings

logger = logging.getLogger(__name__)

_s3_session = aioboto3.Session()


async def upload_screenshot(b64_data: str, key: str) -> str:
    """Decode a base64 PNG and upload it to R2.

    Returns the S3 key on success, or ``""`` if storage is not configured or
    the upload fails.  This is best-effort — it never raises.
    """
    if not settings.s3_bucket or not b64_data:
        return ""

    try:
        data = base64.b64decode(b64_data)

        async with _s3_session.client(
            "s3",
            endpoint_url=settings.s3_endpoint_url or None,
            region_name=settings.s3_region,
        ) as s3:
            await s3.put_object(
                Bucket=settings.s3_bucket,
                Key=key,
                Body=data,
                ContentType="image/png",
            )

        logger.info("Uploaded screenshot %s (%d bytes)", key, len(data))
        return key
    except Exception:
        logger.exception("Failed to upload screenshot %s", key)
        return ""
