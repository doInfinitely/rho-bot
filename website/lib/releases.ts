const GITHUB_RELEASES_URL =
  "https://api.github.com/repos/doInfinitely/rho-bot/releases/latest";

export interface ReleaseAsset {
  name: string;
  url: string;
}

export interface LatestRelease {
  tag: string;
  assets: ReleaseAsset[];
}

export type MacArch = "arm64" | "x64";

export async function fetchLatestRelease(): Promise<LatestRelease> {
  const res = await fetch(GITHUB_RELEASES_URL, {
    headers: {
      Accept: "application/vnd.github+json",
      "User-Agent": "rho-bot-website",
    },
    next: { revalidate: 300 },
  });

  if (!res.ok) {
    throw new Error(`GitHub release fetch failed: ${res.status}`);
  }

  const data = await res.json();

  return {
    tag: data.tag_name,
    assets: (data.assets ?? []).map(
      (asset: { name: string; browser_download_url: string }) => ({
        name: asset.name,
        url: asset.browser_download_url,
      }),
    ),
  };
}

export function findMacAsset(
  assets: ReleaseAsset[],
  preferredArch: MacArch,
): ReleaseAsset | null {
  const dmgAssets = assets.filter((asset) =>
    asset.name.toLowerCase().endsWith(".dmg"),
  );

  const exactMatch = dmgAssets.find((asset) => {
    const name = asset.name.toLowerCase();
    return preferredArch === "arm64"
      ? name.includes("aarch64") || name.includes("arm64")
      : name.includes("x64") || name.includes("x86_64");
  });

  return exactMatch ?? dmgAssets[0] ?? null;
}

export function normalizeMacArch(value: string | null): MacArch {
  if (!value) return "arm64";

  const normalized = value.toLowerCase();
  if (
    normalized === "x64" ||
    normalized === "intel" ||
    normalized === "x86_64"
  ) {
    return "x64";
  }

  return "arm64";
}
