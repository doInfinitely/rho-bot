"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Download, Apple, ArrowRight } from "lucide-react";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

type Platform = "macos" | "windows" | "linux" | "unknown";

const RELEASES_BASE =
  "https://github.com/doInfinitely/rho-bot/releases/latest/download";

/* Hardcoded fallbacks if the API call fails */
const FALLBACK_INSTALLERS: Record<
  Exclude<Platform, "unknown">,
  { label: string; url: string; note: string }
> = {
  macos: {
    label: "Download for macOS",
    url: `${RELEASES_BASE}/rho-bot_0.1.1_aarch64.dmg`,
    note: "macOS 13+ &middot; Apple Silicon",
  },
  windows: {
    label: "Download for Windows",
    url: `${RELEASES_BASE}/rho-bot_0.1.1_x64-setup.exe`,
    note: "Windows 10+ &middot; 64-bit",
  },
  linux: {
    label: "Download for Linux",
    url: `${RELEASES_BASE}/rho-bot_0.1.1_amd64.AppImage`,
    note: "Ubuntu 22.04+ / Fedora 38+ &middot; x86_64",
  },
};

interface PlatformInstaller {
  label: string;
  url: string;
  note: string;
}

function detectPlatform(): Platform {
  if (typeof navigator === "undefined") return "unknown";
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "macos";
  if (ua.includes("win")) return "windows";
  if (ua.includes("linux")) return "linux";
  return "unknown";
}

function matchAssets(
  assets: { name: string; url: string }[],
): Record<Exclude<Platform, "unknown">, PlatformInstaller> | null {
  let macos: PlatformInstaller | null = null;
  let windows: PlatformInstaller | null = null;
  let linux: PlatformInstaller | null = null;

  for (const asset of assets) {
    const name = asset.name.toLowerCase();
    if (name.endsWith(".dmg") && name.includes("aarch64")) {
      macos = {
        label: "Download for macOS",
        url: asset.url,
        note: "macOS 13+ &middot; Apple Silicon",
      };
    } else if (name.endsWith("setup.exe") || name.endsWith(".msi")) {
      windows = {
        label: "Download for Windows",
        url: asset.url,
        note: "Windows 10+ &middot; 64-bit",
      };
    } else if (name.endsWith(".appimage")) {
      linux = {
        label: "Download for Linux",
        url: asset.url,
        note: "Ubuntu 22.04+ / Fedora 38+ &middot; x86_64",
      };
    }
  }

  if (!macos || !windows || !linux) return null;
  return { macos, windows, linux };
}

export default function DownloadPage() {
  const [platform, setPlatform] = useState<Platform>("unknown");
  const [installers, setInstallers] =
    useState<Record<Exclude<Platform, "unknown">, PlatformInstaller>>(
      FALLBACK_INSTALLERS,
    );

  useEffect(() => {
    setPlatform(detectPlatform());

    fetch("/api/releases")
      .then((res) => (res.ok ? res.json() : null))
      .then((data) => {
        if (!data?.assets) return;
        const matched = matchAssets(data.assets);
        if (matched) setInstallers(matched);
      })
      .catch(() => {
        /* keep fallbacks */
      });
  }, []);

  const primary = platform !== "unknown" ? platform : "macos";
  const others = (
    Object.keys(installers) as Exclude<Platform, "unknown">[]
  ).filter((p) => p !== primary);

  return (
    <>
      <Navbar />

      <section className="relative pt-32 pb-24 px-6">
        {/* Gradient glow */}
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[700px] h-[500px] bg-rho-600/10 rounded-full blur-[128px] pointer-events-none" />

        <div className="relative mx-auto max-w-3xl text-center">
          <div className="inline-flex items-center gap-2 px-4 py-1.5 mb-6 rounded-full border border-neutral-800 bg-neutral-900/80 text-xs text-neutral-400">
            <Download className="w-3.5 h-3.5" />
            Desktop App
          </div>

          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1]">
            Install{" "}
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-rho-400 to-rho-600">
              rho-bot
            </span>
          </h1>

          <p className="mt-4 text-lg text-neutral-400 max-w-xl mx-auto leading-relaxed">
            Download the desktop agent to start automating — or record your
            workflow to help train future models.
          </p>

          {/* Primary download */}
          <div className="mt-10">
            <a
              href={installers[primary].url}
              className="inline-flex items-center gap-3 px-8 py-4 text-sm font-medium rounded-xl bg-rho-600 hover:bg-rho-700 transition-colors"
            >
              <Download className="w-5 h-5" />
              {installers[primary].label}
            </a>
            <p
              className="mt-3 text-xs text-neutral-500"
              dangerouslySetInnerHTML={{ __html: installers[primary].note }}
            />
          </div>

          {/* Other platforms */}
          <div className="mt-12 flex flex-col sm:flex-row items-center justify-center gap-4">
            {others.map((p) => (
              <a
                key={p}
                href={installers[p].url}
                className="flex items-center gap-2 px-5 py-2.5 text-sm rounded-lg border border-neutral-800 hover:border-neutral-700 text-neutral-400 hover:text-neutral-200 transition-colors"
              >
                <Download className="w-4 h-4" />
                {installers[p].label}
              </a>
            ))}
          </div>
        </div>
      </section>

      {/* Setup instructions */}
      <section className="py-20 px-6 border-t border-neutral-800/40">
        <div className="mx-auto max-w-2xl">
          <h2 className="text-2xl font-bold tracking-tight text-center mb-10">
            Quick Setup
          </h2>

          <ol className="space-y-6">
            {[
              {
                step: "1",
                title: "Run the installer",
                desc: "Open the downloaded DMG and drag rho-bot to your Applications folder (macOS) or follow the setup wizard (Windows/Linux).",
              },
              {
                step: "2",
                title: "Clear the quarantine flag (macOS)",
                desc: 'If macOS says the app is "damaged," open Terminal and run: xattr -cr /Applications/rho-bot.app — then open the app again.',
              },
              {
                step: "3",
                title: "Grant permissions",
                desc: "rho-bot needs Screen Recording and Accessibility access. The app will prompt you on first launch — approve both in System Settings.",
              },
              {
                step: "4",
                title: "Sign in",
                desc: "Open rho-bot from your menu bar and sign in with your account. The agent connects to the server automatically.",
              },
            ].map(({ step, title, desc }) => (
              <li key={step} className="flex gap-4">
                <span className="flex-shrink-0 w-8 h-8 rounded-lg bg-rho-600/10 text-rho-400 text-sm font-bold flex items-center justify-center">
                  {step}
                </span>
                <div>
                  <h3 className="font-semibold mb-1">{title}</h3>
                  <p className="text-sm text-neutral-500 leading-relaxed">
                    {desc}
                  </p>
                </div>
              </li>
            ))}
          </ol>

          {/* macOS troubleshooting */}
          <div className="mt-10 p-5 rounded-xl border border-amber-800/40 bg-amber-950/20">
            <h3 className="font-semibold text-amber-400 mb-2 flex items-center gap-2">
              <Apple className="w-4 h-4" />
              macOS: &ldquo;App is damaged&rdquo; fix
            </h3>
            <p className="text-sm text-neutral-400 leading-relaxed mb-3">
              macOS quarantines apps downloaded from the internet. Since rho-bot
              is not yet notarized with Apple, Gatekeeper may block it. To fix
              this, open <strong>Terminal</strong> and run:
            </p>
            <code className="block px-4 py-2.5 rounded-lg bg-neutral-900 text-sm text-neutral-200 font-mono">
              xattr -cr /Applications/rho-bot.app
            </code>
            <p className="text-xs text-neutral-500 mt-2">
              Then open the app normally. You may also need to right-click
              &rarr; Open the first time.
            </p>
          </div>

          <div className="mt-10 text-center">
            <p className="text-sm text-neutral-500 mb-4">
              Don&apos;t have an account yet?
            </p>
            <Link
              href="/signup"
              className="inline-flex items-center gap-2 text-sm text-rho-400 hover:text-rho-300 transition-colors"
            >
              Create a free account
              <ArrowRight className="w-3.5 h-3.5" />
            </Link>
          </div>
        </div>
      </section>

      <Footer />
    </>
  );
}
