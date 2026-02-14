"use client";

import Link from "next/link";
import { Check } from "lucide-react";
import { cn } from "@/lib/utils";
import type { PricingTier } from "@/lib/pricing";

interface PricingCardProps {
  tier: PricingTier;
  annual?: boolean;
}

export default function PricingCard({ tier, annual = false }: PricingCardProps) {
  const price = annual ? tier.annualPrice : tier.monthlyPrice;
  const isCustom = price === null;

  return (
    <div
      className={cn(
        "relative flex flex-col rounded-2xl border p-8 transition-all duration-200",
        tier.highlighted
          ? "border-rho-500/50 bg-rho-950/20 shadow-lg shadow-rho-500/5 scale-[1.02]"
          : "border-neutral-800 bg-neutral-900/50 hover:border-neutral-700"
      )}
    >
      {tier.highlighted && (
        <div className="absolute -top-3.5 left-1/2 -translate-x-1/2">
          <span className="px-4 py-1 text-xs font-semibold rounded-full bg-rho-600 text-white">
            Most Popular
          </span>
        </div>
      )}

      <div className="mb-6">
        <h3 className="text-lg font-semibold text-neutral-100">{tier.name}</h3>
        <p className="mt-1 text-sm text-neutral-500">{tier.description}</p>
      </div>

      <div className="mb-6">
        {isCustom ? (
          <div className="flex items-baseline gap-1">
            <span className="text-4xl font-bold tracking-tight text-neutral-100">Custom</span>
          </div>
        ) : (
          <div className="flex items-baseline gap-1">
            <span className="text-4xl font-bold tracking-tight text-neutral-100">
              ${price}
            </span>
            {price > 0 && (
              <span className="text-sm text-neutral-500">/ month</span>
            )}
          </div>
        )}
        {tier.effectiveRate && (
          <p className="mt-1 text-xs text-neutral-500">{tier.effectiveRate}</p>
        )}
        {annual && !isCustom && price !== undefined && price > 0 && (
          <p className="mt-1 text-xs text-rho-400">
            Billed annually (${price * 12}/yr)
          </p>
        )}
      </div>

      <div className="mb-8">
        <div className="inline-flex items-center px-3 py-1 rounded-full bg-neutral-800/80 text-xs text-neutral-300">
          {tier.taskLimit}
        </div>
      </div>

      <ul className="mb-8 flex-1 space-y-3">
        {tier.features.map((feature) => (
          <li key={feature} className="flex items-start gap-3">
            <Check className="w-4 h-4 mt-0.5 shrink-0 text-rho-400" />
            <span className="text-sm text-neutral-400">{feature}</span>
          </li>
        ))}
      </ul>

      <Link
        href={tier.ctaHref}
        className={cn(
          "block w-full text-center py-3 px-4 text-sm font-medium rounded-lg transition-colors",
          tier.highlighted
            ? "bg-rho-600 hover:bg-rho-700 text-white"
            : "bg-neutral-800 hover:bg-neutral-700 text-neutral-200"
        )}
      >
        {tier.cta}
      </Link>
    </div>
  );
}
