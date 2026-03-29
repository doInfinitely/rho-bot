"use client";

import Link from "next/link";
import {
  Zap,
  Shield,
  Brain,
  Layers,
  ArrowRight,
  Download,
  Monitor,
  Bot,
  Target,
} from "lucide-react";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import PricingCard from "@/components/PricingCard";

export default function HomePage() {

  return (
    <>
      <Navbar />

      {/* ── Hero ── */}
      <section className="relative pt-32 pb-20 px-6 overflow-hidden">
        {/* Gradient glow */}
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[600px] bg-rho-600/10 rounded-full blur-[128px] pointer-events-none" />

        <div className="relative mx-auto max-w-4xl text-center">
          <div className="inline-flex items-center gap-2 px-4 py-1.5 mb-6 rounded-full border border-neutral-800 bg-neutral-900/80 text-xs text-neutral-400">
            <span className="w-1.5 h-1.5 rounded-full bg-rho-400 animate-pulse" />
            Now in early access
          </div>

          <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold tracking-tight leading-[1.1]">
            Your computer,{" "}
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-rho-400 to-rho-600">
              on autopilot
            </span>
          </h1>

          <p className="mt-6 text-lg sm:text-xl text-neutral-400 max-w-2xl mx-auto leading-relaxed">
            rho-bot is an autonomous desktop agent that observes how you work,
            learns your goals, and executes tasks — clicks, keystrokes, and all.
            10–50x cheaper than LLM-based computer use.
          </p>

          <div className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              href="/download"
              className="w-full sm:w-auto px-8 py-3.5 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 transition-colors flex items-center justify-center gap-2"
            >
              <Download className="w-4 h-4" />
              Download App
            </Link>
            <Link
              href="/signup"
              className="w-full sm:w-auto px-8 py-3.5 text-sm font-medium rounded-lg border border-neutral-700 hover:border-neutral-600 text-neutral-300 hover:text-neutral-100 transition-colors flex items-center justify-center gap-2"
            >
              Get Started Free
              <ArrowRight className="w-4 h-4" />
            </Link>
          </div>

          <p className="mt-4 text-xs text-neutral-600">
            No credit card required &middot; 25 free tasks per month
          </p>
        </div>
      </section>

      {/* ── Logos / Social proof ── */}
      <section className="py-12 border-y border-neutral-800/40">
        <div className="mx-auto max-w-4xl px-6 text-center">
          <p className="text-xs uppercase tracking-widest text-neutral-600 mb-6">
            Built on proven research
          </p>
          <div className="flex items-center justify-center gap-10 flex-wrap text-neutral-600">
            <span className="text-sm font-medium">Hierarchical Goal Induction</span>
            <span className="text-neutral-800">|</span>
            <span className="text-sm font-medium">Distilled Action Models</span>
            <span className="text-neutral-800">|</span>
            <span className="text-sm font-medium">Preference-Guided Search</span>
          </div>
        </div>
      </section>

      {/* ── Features ── */}
      <section className="py-24 px-6">
        <div className="mx-auto max-w-6xl">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
              Why rho-bot?
            </h2>
            <p className="mt-4 text-neutral-400 max-w-2xl mx-auto">
              Purpose-built desktop automation that actually understands what
              you're trying to do.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              {
                icon: Zap,
                title: "10–50x Cheaper",
                desc: "Distilled models, not frontier LLMs. A task that costs $1–3 on Anthropic costs pennies here.",
              },
              {
                icon: Brain,
                title: "Understands Goals",
                desc: "Hierarchical goal induction detects nested plans — not just next-token prediction.",
              },
              {
                icon: Shield,
                title: "Runs Locally",
                desc: "Your screen data stays on your machine. The agent model can run fully on-device.",
              },
              {
                icon: Layers,
                title: "Gets Better",
                desc: "Preference-guided search + distillation means the agent improves over time.",
              },
            ].map(({ icon: Icon, title, desc }) => (
              <div
                key={title}
                className="p-6 rounded-xl border border-neutral-800 bg-neutral-900/50 hover:border-neutral-700 transition-colors"
              >
                <div className="w-10 h-10 rounded-lg bg-rho-600/10 flex items-center justify-center mb-4">
                  <Icon className="w-5 h-5 text-rho-400" />
                </div>
                <h3 className="font-semibold mb-2">{title}</h3>
                <p className="text-sm text-neutral-500 leading-relaxed">{desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ── How It Works ── */}
      <section id="how-it-works" className="py-24 px-6 bg-neutral-900/30">
        <div className="mx-auto max-w-5xl">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
              How It Works
            </h2>
            <p className="mt-4 text-neutral-400 max-w-2xl mx-auto">
              Three simple steps from installation to automation.
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                icon: Monitor,
                step: "01",
                title: "Install the Agent",
                desc: "Download the rho-bot desktop app. It captures your screen and accessibility tree securely on your device.",
                link: "/download",
              },
              {
                icon: Target,
                step: "02",
                title: "Describe Your Goal",
                desc: "Tell the agent what you want — 'book a flight to Paris' or 'organize my downloads folder.'",
              },
              {
                icon: Bot,
                step: "03",
                title: "Watch It Work",
                desc: "The agent decomposes your goal into a hierarchical plan and executes it through real clicks and keystrokes.",
              },
            ].map(({ icon: Icon, step, title, desc, link }) => (
              <div key={step} className="relative">
                <span className="text-6xl font-bold text-neutral-800/50 absolute -top-4 -left-2">
                  {step}
                </span>
                <div className="relative pt-10">
                  <div className="w-10 h-10 rounded-lg bg-rho-600/10 flex items-center justify-center mb-4">
                    <Icon className="w-5 h-5 text-rho-400" />
                  </div>
                  <h3 className="font-semibold mb-2">{title}</h3>
                  <p className="text-sm text-neutral-500 leading-relaxed">{desc}</p>
                  {link && (
                    <Link
                      href={link}
                      className="mt-3 inline-flex items-center gap-1.5 text-sm text-rho-400 hover:text-rho-300 transition-colors"
                    >
                      <Download className="w-3.5 h-3.5" />
                      Download now
                    </Link>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ── Pricing preview ── */}
      <section className="py-24 px-6">
        <div className="mx-auto max-w-3xl">
          <div className="text-center mb-12">
            <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
              Simple, Fair Pricing
            </h2>
            <p className="mt-4 text-neutral-400 max-w-2xl mx-auto">
              Start free. Upgrade to Pro for unlimited tasks.
            </p>
          </div>

          <PricingCard />

          <div className="mt-10 text-center">
            <Link
              href="/pricing"
              className="text-sm text-rho-400 hover:text-rho-300 transition-colors inline-flex items-center gap-1"
            >
              Learn more about pricing
              <ArrowRight className="w-3.5 h-3.5" />
            </Link>
          </div>
        </div>
      </section>

      {/* ── CTA ── */}
      <section className="py-24 px-6">
        <div className="mx-auto max-w-3xl text-center">
          <div className="p-12 rounded-2xl border border-neutral-800 bg-gradient-to-b from-rho-950/30 to-neutral-900/50">
            <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
              Ready to automate?
            </h2>
            <p className="mt-4 text-neutral-400 max-w-lg mx-auto">
              Start with 25 free tasks per month. No credit card required.
            </p>
            <div className="mt-8">
              <Link
                href="/signup"
                className="inline-flex items-center gap-2 px-8 py-3.5 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 transition-colors"
              >
                Get Started Free
                <ArrowRight className="w-4 h-4" />
              </Link>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </>
  );
}
