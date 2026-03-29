import Link from "next/link";
import { Check } from "lucide-react";
import { FREE_FEATURES, PRO_FEATURES, PRO_PRICE } from "@/lib/pricing";

export default function PricingCard() {
  return (
    <div className="grid md:grid-cols-2 gap-6 max-w-3xl mx-auto w-full">
      {/* Free */}
      <div className="flex flex-col rounded-2xl border border-neutral-800 bg-neutral-900/50 p-8">
        <div className="mb-6">
          <h3 className="text-lg font-semibold text-neutral-100">Free</h3>
          <p className="mt-1 text-sm text-neutral-500">
            Get started with 25 tasks per month.
          </p>
        </div>

        <div className="mb-6">
          <div className="flex items-baseline gap-1">
            <span className="text-5xl font-bold tracking-tight text-neutral-100">
              $0
            </span>
            <span className="text-sm text-neutral-500">/ month</span>
          </div>
        </div>

        <ul className="mb-8 flex-1 space-y-3">
          {FREE_FEATURES.map((feature) => (
            <li key={feature} className="flex items-start gap-3">
              <Check className="w-4 h-4 mt-0.5 shrink-0 text-neutral-500" />
              <span className="text-sm text-neutral-400">{feature}</span>
            </li>
          ))}
        </ul>

        <Link
          href="/signup"
          className="block w-full text-center py-3 px-4 text-sm font-medium rounded-lg border border-neutral-700 hover:border-neutral-600 text-neutral-300 hover:text-neutral-100 transition-colors"
        >
          Get Started Free
        </Link>
      </div>

      {/* Pro */}
      <div className="relative flex flex-col rounded-2xl border border-rho-500/50 bg-rho-950/20 shadow-lg shadow-rho-500/5 p-8">
        <div className="absolute -top-3.5 left-1/2 -translate-x-1/2">
          <span className="px-4 py-1 text-xs font-semibold rounded-full bg-rho-600 text-white">
            Most Popular
          </span>
        </div>

        <div className="mb-6">
          <h3 className="text-lg font-semibold text-neutral-100">Pro</h3>
          <p className="mt-1 text-sm text-neutral-500">
            Unlimited tasks for power users.
          </p>
        </div>

        <div className="mb-6">
          <div className="flex items-baseline gap-1">
            <span className="text-5xl font-bold tracking-tight text-neutral-100">
              ${PRO_PRICE}
            </span>
            <span className="text-sm text-neutral-500">/ month</span>
          </div>
        </div>

        <ul className="mb-8 flex-1 space-y-3">
          {PRO_FEATURES.map((feature) => (
            <li key={feature} className="flex items-start gap-3">
              <Check className="w-4 h-4 mt-0.5 shrink-0 text-rho-400" />
              <span className="text-sm text-neutral-400">{feature}</span>
            </li>
          ))}
        </ul>

        <Link
          href="/signup?plan=pro"
          className="block w-full text-center py-3 px-4 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 text-white transition-colors"
        >
          Start Pro
        </Link>
      </div>
    </div>
  );
}
