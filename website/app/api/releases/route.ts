import { NextResponse } from "next/server";
import { fetchLatestRelease, findMacAsset } from "@/lib/releases";

export const revalidate = 300; // cache for 5 minutes

export async function GET() {
  try {
    const release = await fetchLatestRelease();

    return NextResponse.json({
      tag: release.tag,
      assets: release.assets,
      macos: {
        arm64: findMacAsset(release.assets, "arm64"),
        x64: findMacAsset(release.assets, "x64"),
      },
    });
  } catch {
    return NextResponse.json(
      { error: "Failed to fetch release" },
      { status: 502 },
    );
  }
}
