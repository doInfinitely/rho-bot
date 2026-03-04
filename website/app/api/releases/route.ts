import { NextResponse } from "next/server";

const GITHUB_RELEASES_URL =
  "https://api.github.com/repos/doInfinitely/rho-bot/releases/latest";

export const revalidate = 300; // cache for 5 minutes

export async function GET() {
  try {
    const res = await fetch(GITHUB_RELEASES_URL, {
      headers: { Accept: "application/vnd.github+json" },
      next: { revalidate: 300 },
    });

    if (!res.ok) {
      return NextResponse.json(
        { error: "Failed to fetch release" },
        { status: res.status },
      );
    }

    const data = await res.json();

    const assets = (data.assets ?? []).map(
      (a: { name: string; browser_download_url: string }) => ({
        name: a.name,
        url: a.browser_download_url,
      }),
    );

    return NextResponse.json({ tag: data.tag_name, assets });
  } catch {
    return NextResponse.json(
      { error: "Failed to fetch release" },
      { status: 502 },
    );
  }
}
