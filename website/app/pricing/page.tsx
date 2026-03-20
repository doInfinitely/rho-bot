"use client";

import { useState } from "react";
import Link from "next/link";
import { ArrowRight } from "lucide-react";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import PricingCard from "@/components/PricingCard";
import { FAQ_ITEMS } from "@/lib/pricing";

export default function PricingPage() {
  const [openFaq, setOpenFaq] = useState<number | null>(null);

  return (
    <>
      <Navbar />

      {/* ── Header ── */}
      <section className="relative pt-32 pb-16 px-6">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[600px] h-[400px] bg-rho-600/8 rounded-full blur-[128px] pointer-events-none" />

        <div className="relative mx-auto max-w-4xl text-center">
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight">
            Pay What You Want
          </h1>
          <p className="mt-4 text-lg text-neutral-400 max-w-2xl mx-auto">
            Full access to everything. Pay what it's worth to you.
          </p>
        </div>
      </section>

      {/* ── Pricing card ── */}
      <section className="px-6 pb-24">
        <div className="mx-auto max-w-lg">
          <PricingCard />
        </div>
      </section>

      {/* ── Cost comparison ── */}
      <section className="py-24 px-6 bg-neutral-900/30">
        <div className="mx-auto max-w-4xl">
          <h2 className="text-2xl sm:text-3xl font-bold tracking-tight text-center mb-4">
            Cost Comparison
          </h2>
          <p className="text-center text-neutral-400 mb-12 max-w-2xl mx-auto">
            See how rho-bot stacks up against LLM-based computer use.
          </p>

          <div className="grid md:grid-cols-3 gap-6">
            {/* Anthropic */}
            <div className="p-6 rounded-xl border border-neutral-800 bg-neutral-900/50">
              <h3 className="font-semibold text-neutral-300 mb-1">
                Anthropic Computer Use
              </h3>
              <p className="text-xs text-neutral-500 mb-4">
                Claude Sonnet — per-token billing
              </p>
              <div className="text-3xl font-bold text-red-400">
                $250–$1,500
              </div>
              <p className="text-xs text-neutral-500 mt-1">
                / month for 500 tasks
              </p>
            </div>

            {/* OpenAI */}
            <div className="p-6 rounded-xl border border-neutral-800 bg-neutral-900/50">
              <h3 className="font-semibold text-neutral-300 mb-1">
                OpenAI Operator
              </h3>
              <p className="text-xs text-neutral-500 mb-4">
                Container + token billing
              </p>
              <div className="text-3xl font-bold text-orange-400">
                $150–$800
              </div>
              <p className="text-xs text-neutral-500 mt-1">
                / month for 500 tasks
              </p>
            </div>

            {/* rho-bot */}
            <div className="p-6 rounded-xl border border-rho-500/40 bg-rho-950/20">
              <h3 className="font-semibold text-neutral-200 mb-1">rho-bot</h3>
              <p className="text-xs text-neutral-500 mb-4">
                Pay what you want — distilled models
              </p>
              <div className="text-3xl font-bold text-rho-400">$0+</div>
              <p className="text-xs text-neutral-500 mt-1">
                / month — you decide
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* ── FAQ ── */}
      <section className="py-24 px-6">
        <div className="mx-auto max-w-3xl">
          <h2 className="text-2xl sm:text-3xl font-bold tracking-tight text-center mb-12">
            Frequently Asked Questions
          </h2>

          <div className="space-y-3">
            {FAQ_ITEMS.map((item, i) => (
              <div
                key={i}
                className="border border-neutral-800 rounded-xl overflow-hidden"
              >
                <button
                  onClick={() => setOpenFaq(openFaq === i ? null : i)}
                  className="w-full text-left px-6 py-4 flex items-center justify-between gap-4 hover:bg-neutral-900/50 transition-colors"
                >
                  <span className="text-sm font-medium text-neutral-200">
                    {item.question}
                  </span>
                  <svg
                    className={`w-4 h-4 shrink-0 text-neutral-500 transition-transform ${
                      openFaq === i ? "rotate-180" : ""
                    }`}
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M19 9l-7 7-7-7"
                    />
                  </svg>
                </button>
                {openFaq === i && (
                  <div className="px-6 pb-4">
                    <p className="text-sm text-neutral-400 leading-relaxed">
                      {item.answer}
                    </p>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ── CTA ── */}
      <section className="py-24 px-6">
        <div className="mx-auto max-w-3xl text-center">
          <div className="p-12 rounded-2xl border border-neutral-800 bg-gradient-to-b from-rho-950/30 to-neutral-900/50">
            <h2 className="text-3xl font-bold tracking-tight">
              Start automating today
            </h2>
            <p className="mt-4 text-neutral-400 max-w-lg mx-auto">
              Free to use. Pay what you want when you're ready.
            </p>
            <div className="mt-8">
              <Link
                href="/signup"
                className="inline-flex items-center justify-center gap-2 px-8 py-3.5 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 transition-colors"
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
