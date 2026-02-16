"""
AES-256-GCM encryption and decryption for PyTorch model weights.

Encrypt weights offline before deployment; decrypt in-memory at server
startup so plaintext never touches disk in production.

CLI usage::

    # Generate a key
    python -c "import os; print(os.urandom(32).hex())"

    # Encrypt
    python -m server.services.model_security encrypt model.pt model.pt.enc --key <hex>

    # Decrypt (for verification only -- the server does this in memory)
    python -m server.services.model_security decrypt model.pt.enc model_out.pt --key <hex>
"""

from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

from cryptography.hazmat.primitives.ciphers.aead import AESGCM

logger = logging.getLogger(__name__)

_NONCE_BYTES = 12  # 96-bit nonce recommended for AES-GCM


def encrypt_file(plaintext_path: str | Path, output_path: str | Path, key_hex: str) -> None:
    """Encrypt a file with AES-256-GCM and write *nonce || ciphertext* to *output_path*."""
    key = bytes.fromhex(key_hex)
    if len(key) != 32:
        raise ValueError("Key must be 32 bytes (64 hex characters)")

    import os
    nonce = os.urandom(_NONCE_BYTES)
    aesgcm = AESGCM(key)

    plaintext = Path(plaintext_path).read_bytes()
    ciphertext = aesgcm.encrypt(nonce, plaintext, None)

    Path(output_path).write_bytes(nonce + ciphertext)
    logger.info("Encrypted %s -> %s (%d bytes)", plaintext_path, output_path, len(nonce + ciphertext))


def decrypt_file(encrypted_path: str | Path, key_hex: str) -> bytes:
    """Decrypt a *nonce || ciphertext* file in memory and return the plaintext bytes."""
    key = bytes.fromhex(key_hex)
    if len(key) != 32:
        raise ValueError("Key must be 32 bytes (64 hex characters)")

    raw = Path(encrypted_path).read_bytes()
    if len(raw) <= _NONCE_BYTES:
        raise ValueError("Encrypted file is too small to contain a valid nonce + ciphertext")

    nonce = raw[:_NONCE_BYTES]
    ciphertext = raw[_NONCE_BYTES:]

    aesgcm = AESGCM(key)
    plaintext = aesgcm.decrypt(nonce, ciphertext, None)
    logger.info("Decrypted %s in memory (%d bytes)", encrypted_path, len(plaintext))
    return plaintext


def decrypt_file_to_disk(encrypted_path: str | Path, output_path: str | Path, key_hex: str) -> None:
    """Decrypt and write to disk (for offline verification only)."""
    plaintext = decrypt_file(encrypted_path, key_hex)
    Path(output_path).write_bytes(plaintext)
    logger.info("Wrote decrypted output to %s", output_path)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        description="Encrypt or decrypt PyTorch model weights with AES-256-GCM.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    enc = sub.add_parser("encrypt", help="Encrypt a .pt file")
    enc.add_argument("input", help="Path to plaintext .pt file")
    enc.add_argument("output", help="Path for encrypted output (e.g. model.pt.enc)")
    enc.add_argument("--key", required=True, help="Hex-encoded 32-byte AES key")

    dec = sub.add_parser("decrypt", help="Decrypt a .pt.enc file (for verification)")
    dec.add_argument("input", help="Path to encrypted file")
    dec.add_argument("output", help="Path for decrypted output")
    dec.add_argument("--key", required=True, help="Hex-encoded 32-byte AES key")

    args = parser.parse_args(argv)

    logging.basicConfig(level=logging.INFO, format="%(message)s")

    if args.command == "encrypt":
        encrypt_file(args.input, args.output, args.key)
    elif args.command == "decrypt":
        decrypt_file_to_disk(args.input, args.output, args.key)


if __name__ == "__main__":
    main()
