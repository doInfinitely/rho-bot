import Link from "next/link";
import { Download, Apple, ArrowRight } from "lucide-react";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

export default function DownloadPage() {
  return (
    <>
      <Navbar />

      <section className="relative pt-32 pb-24 px-6">
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
            Download the macOS desktop agent to start automating, or record
            your workflow to help train future models.
          </p>

          <div className="mt-6 inline-flex items-center gap-2 rounded-full border border-rho-500/20 bg-rho-500/10 px-4 py-1.5 text-xs text-rho-300">
            macOS only
          </div>

          <div className="mt-10 flex flex-col items-center gap-4">
            <a
              href="/api/releases/download?arch=arm64"
              className="inline-flex items-center gap-3 px-8 py-4 text-sm font-medium rounded-xl bg-rho-600 hover:bg-rho-700 transition-colors"
            >
              <Download className="w-5 h-5" />
              Download for Apple Silicon
            </a>
            <p className="text-xs text-neutral-500">
              macOS 13+ for Apple Silicon Macs. Served from the latest GitHub
              release.
            </p>

            <a
              href="/api/releases/download?arch=x64"
              className="flex items-center gap-2 px-5 py-2.5 text-sm rounded-lg border border-neutral-800 hover:border-neutral-700 text-neutral-400 hover:text-neutral-200 transition-colors"
            >
              <Download className="w-4 h-4" />
              Download for Intel Mac
            </a>
            <p className="text-xs text-neutral-500">
              Use this build if your Mac has an Intel processor.
            </p>
          </div>
        </div>
      </section>

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
                desc: "Open the downloaded DMG and drag rho-bot to your Applications folder.",
              },
              {
                step: "2",
                title: "Clear the quarantine flag (macOS)",
                desc: 'If macOS says the app is "damaged," open Terminal and run: xattr -cr /Applications/rho-bot.app, then open the app again.',
              },
              {
                step: "3",
                title: "Grant permissions",
                desc: "rho-bot needs Screen Recording and Accessibility access. The app will prompt you on first launch, approve both in System Settings.",
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
