import { NextRequest, NextResponse } from "next/server";
import {
  fetchLatestRelease,
  findMacAsset,
  normalizeMacArch,
} from "@/lib/releases";

export const revalidate = 300;

export async function GET(request: NextRequest) {
  try {
    const arch = normalizeMacArch(
      request.nextUrl.searchParams.get("arch"),
    );
    const release = await fetchLatestRelease();
    const asset = findMacAsset(release.assets, arch);

    if (!asset) {
      return NextResponse.json(
        { error: "No macOS installer found in the latest release" },
        { status: 404 },
      );
    }

    return NextResponse.redirect(asset.url);
  } catch {
    return NextResponse.json(
      { error: "Failed to resolve latest macOS download" },
      { status: 502 },
    );
  }
}
