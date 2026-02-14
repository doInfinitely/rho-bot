"use client";

import { useState, Fragment } from "react";
import Link from "next/link";
import { ArrowRight, Check, Minus } from "lucide-react";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import PricingCard from "@/components/PricingCard";
import { PRICING_TIERS, FAQ_ITEMS } from "@/lib/pricing";

/* ── Feature comparison data ── */
const COMPARISON_CATEGORIES = [
  {
    name: "Usage",
    features: [
      { label: "Tasks per month", values: ["50", "500", "2,000 / seat", "Unlimited"] },
      { label: "Concurrent sessions", values: ["1", "3", "10", "Unlimited"] },
      { label: "Session history", values: ["7 days", "90 days", "Unlimited", "Unlimited"] },
      { label: "Overage rate", values: ["—", "$0.10 / task", "$0.10 / task", "N/A"] },
    ],
  },
  {
    name: "Features",
    features: [
      { label: "Standard execution", values: [true, true, true, true] },
      { label: "Priority execution", values: [false, true, true, true] },
      { label: "API access", values: [false, true, true, true] },
      { label: "Custom task templates", values: [false, true, true, true] },
      { label: "Team workspace", values: [false, false, true, true] },
      { label: "Role-based access control", values: [false, false, true, true] },
      { label: "On-premise deployment", values: [false, false, false, true] },
      { label: "Custom model fine-tuning", values: [false, false, false, true] },
    ],
  },
  {
    name: "Support",
    features: [
      { label: "Community support", values: [true, true, true, true] },
      { label: "Email support", values: [false, true, true, true] },
      { label: "Priority SLA", values: [false, false, true, true] },
      { label: "Dedicated engineer", values: [false, false, false, true] },
    ],
  },
];

export default function PricingPage() {
  const [annual, setAnnual] = useState(false);
  const [openFaq, setOpenFaq] = useState<number | null>(null);

  return (
    <>
      <Navbar />

      {/* ── Header ── */}
      <section className="relative pt-32 pb-16 px-6">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[600px] h-[400px] bg-rho-600/8 rounded-full blur-[128px] pointer-events-none" />

        <div className="relative mx-auto max-w-4xl text-center">
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight">
            Pricing
          </h1>
          <p className="mt-4 text-lg text-neutral-400 max-w-2xl mx-auto">
            Start free. Scale when you're ready. 10–50x cheaper than
            LLM-based computer use.
          </p>

          {/* Toggle */}
          <div className="mt-8 inline-flex items-center gap-3 p-1 rounded-full bg-neutral-900 border border-neutral-800">
            <button
              onClick={() => setAnnual(false)}
              className={`px-4 py-1.5 text-sm rounded-full transition-colors ${
                !annual
                  ? "bg-neutral-800 text-neutral-100"
                  : "text-neutral-500 hover:text-neutral-300"
              }`}
            >
              Monthly
            </button>
            <button
              onClick={() => setAnnual(true)}
              className={`px-4 py-1.5 text-sm rounded-full transition-colors ${
                annual
                  ? "bg-neutral-800 text-neutral-100"
                  : "text-neutral-500 hover:text-neutral-300"
              }`}
            >
              Annual
              <span className="ml-1.5 text-xs text-rho-400">save 20%</span>
            </button>
          </div>
        </div>
      </section>

      {/* ── Pricing cards ── */}
      <section className="px-6 pb-24">
        <div className="mx-auto max-w-6xl grid md:grid-cols-2 lg:grid-cols-4 gap-6">
          {PRICING_TIERS.map((tier) => (
            <PricingCard key={tier.id} tier={tier} annual={annual} />
          ))}
        </div>
      </section>

      {/* ── Feature comparison table ── */}
      <section className="py-24 px-6 bg-neutral-900/30">
        <div className="mx-auto max-w-5xl">
          <h2 className="text-2xl sm:text-3xl font-bold tracking-tight text-center mb-12">
            Compare Plans
          </h2>

          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              {/* Header */}
              <thead>
                <tr className="border-b border-neutral-800">
                  <th className="text-left py-4 pr-8 font-normal text-neutral-500 w-1/3">
                    &nbsp;
                  </th>
                  {PRICING_TIERS.map((tier) => (
                    <th
                      key={tier.id}
                      className="text-center py-4 px-4 font-semibold text-neutral-200"
                    >
                      {tier.name}
                    </th>
                  ))}
                </tr>
              </thead>

              <tbody>
                {COMPARISON_CATEGORIES.map((category) => (
                  <Fragment key={category.name}>
                    {/* Category header */}
                    <tr>
                      <td
                        colSpan={5}
                        className="pt-8 pb-3 text-xs uppercase tracking-widest text-neutral-500 font-semibold"
                      >
                        {category.name}
                      </td>
                    </tr>

                    {/* Feature rows */}
                    {category.features.map((feature) => (
                      <tr
                        key={feature.label}
                        className="border-b border-neutral-800/50"
                      >
                        <td className="py-3 pr-8 text-neutral-400">
                          {feature.label}
                        </td>
                        {feature.values.map((value, i) => (
                          <td key={i} className="py-3 px-4 text-center">
                            {typeof value === "boolean" ? (
                              value ? (
                                <Check className="w-4 h-4 text-rho-400 mx-auto" />
                              ) : (
                                <Minus className="w-4 h-4 text-neutral-700 mx-auto" />
                              )
                            ) : (
                              <span className="text-neutral-300">{value}</span>
                            )}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </Fragment>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* ── Cost comparison ── */}
      <section className="py-24 px-6">
        <div className="mx-auto max-w-4xl">
          <h2 className="text-2xl sm:text-3xl font-bold tracking-tight text-center mb-4">
            Cost Comparison
          </h2>
          <p className="text-center text-neutral-400 mb-12 max-w-2xl mx-auto">
            See how rho-bot stacks up against LLM-based computer use for a
            typical workload of 500 tasks per month.
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
              <p className="text-xs text-neutral-600 mt-3">
                $0.50–$3.00 per task (screenshots + reasoning tokens)
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
              <p className="text-xs text-neutral-600 mt-3">
                Container fees + $0.03–$1.92 per session + tokens
              </p>
            </div>

            {/* rho-bot */}
            <div className="p-6 rounded-xl border border-rho-500/40 bg-rho-950/20">
              <h3 className="font-semibold text-neutral-200 mb-1">
                rho-bot Pro
              </h3>
              <p className="text-xs text-neutral-500 mb-4">
                Flat subscription — distilled models
              </p>
              <div className="text-3xl font-bold text-rho-400">$25</div>
              <p className="text-xs text-neutral-500 mt-1">
                / month for 500 tasks
              </p>
              <p className="text-xs text-rho-400/70 mt-3">
                ~$0.05 per task — 10–50x cheaper
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* ── FAQ ── */}
      <section className="py-24 px-6 bg-neutral-900/30">
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
              50 free tasks per month. No credit card needed. Upgrade when
              you're ready.
            </p>
            <div className="mt-8 flex flex-col sm:flex-row items-center justify-center gap-4">
              <Link
                href="/signup"
                className="w-full sm:w-auto inline-flex items-center justify-center gap-2 px-8 py-3.5 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 transition-colors"
              >
                Get Started Free
                <ArrowRight className="w-4 h-4" />
              </Link>
              <a
                href="mailto:sales@rho-bot.dev"
                className="w-full sm:w-auto inline-flex items-center justify-center px-8 py-3.5 text-sm font-medium rounded-lg border border-neutral-700 hover:border-neutral-600 text-neutral-300 hover:text-neutral-100 transition-colors"
              >
                Talk to Sales
              </a>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </>
  );
}
