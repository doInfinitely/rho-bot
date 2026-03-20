"use client";

import { useState } from "react";
import Link from "next/link";
import { Check } from "lucide-react";
import { DEFAULT_AMOUNT, MIN_AMOUNT, FEATURES } from "@/lib/pricing";

export default function PricingCard() {
  const [amount, setAmount] = useState(DEFAULT_AMOUNT);

  return (
    <div className="relative flex flex-col rounded-2xl border border-rho-500/50 bg-rho-950/20 shadow-lg shadow-rho-500/5 p-8 max-w-md mx-auto w-full">
      <div className="absolute -top-3.5 left-1/2 -translate-x-1/2">
        <span className="px-4 py-1 text-xs font-semibold rounded-full bg-rho-600 text-white">
          Pay What You Want
        </span>
      </div>

      <div className="mb-6 text-center">
        <h3 className="text-lg font-semibold text-neutral-100">rho-bot</h3>
        <p className="mt-1 text-sm text-neutral-500">
          Full access. Pay what it's worth to you.
        </p>
      </div>

      {/* Amount input */}
      <div className="mb-6 flex flex-col items-center">
        <div className="flex items-baseline gap-1">
          <span className="text-2xl text-neutral-400">$</span>
          <input
            type="number"
            min={MIN_AMOUNT}
            step={1}
            value={amount}
            onChange={(e) => setAmount(Math.max(MIN_AMOUNT, Number(e.target.value)))}
            className="w-24 text-5xl font-bold tracking-tight text-neutral-100 bg-transparent border-b-2 border-neutral-700 focus:border-rho-500 outline-none text-center appearance-none [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
          />
          <span className="text-sm text-neutral-500">/ month</span>
        </div>
        <p className="mt-2 text-xs text-neutral-600">
          {amount === 0 ? "Free forever" : `$${amount * 12}/yr billed monthly`}
        </p>
      </div>

      {/* Quick-pick buttons */}
      <div className="mb-8 flex justify-center gap-2">
        {[0, 5, 10, 25].map((v) => (
          <button
            key={v}
            onClick={() => setAmount(v)}
            className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
              amount === v
                ? "bg-rho-600 text-white"
                : "bg-neutral-800 text-neutral-400 hover:bg-neutral-700"
            }`}
          >
            {v === 0 ? "Free" : `$${v}`}
          </button>
        ))}
      </div>

      <ul className="mb-8 flex-1 space-y-3">
        {FEATURES.map((feature) => (
          <li key={feature} className="flex items-start gap-3">
            <Check className="w-4 h-4 mt-0.5 shrink-0 text-rho-400" />
            <span className="text-sm text-neutral-400">{feature}</span>
          </li>
        ))}
      </ul>

      <Link
        href={amount > 0 ? `/signup?amount=${amount}` : "/signup"}
        className="block w-full text-center py-3 px-4 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 text-white transition-colors"
      >
        {amount > 0 ? `Get Started — $${amount}/mo` : "Get Started Free"}
      </Link>
    </div>
  );
}
