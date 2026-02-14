"use client";

import { useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import { Check, CreditCard, AlertCircle, ArrowRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { PRICING_TIERS } from "@/lib/pricing";
import {
  getSubscription,
  createCheckoutSession,
  createBillingPortalSession,
} from "@/lib/api";
import type { Subscription } from "@/lib/api";

export default function BillingPage() {
  const searchParams = useSearchParams();
  const suggestedPlan = searchParams.get("plan");

  const [sub, setSub] = useState<Subscription | null>(null);
  const [loading, setLoading] = useState(true);
  const [checkoutLoading, setCheckoutLoading] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const subscription = await getSubscription();
        setSub(subscription);
      } catch {
        // No subscription yet — that's fine
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  async function handleCheckout(planId: string) {
    setCheckoutLoading(planId);
    try {
      const { url } = await createCheckoutSession(planId);
      window.location.href = url;
    } catch (err: any) {
      alert(err.message || "Failed to create checkout session");
    } finally {
      setCheckoutLoading(null);
    }
  }

  async function handleManageBilling() {
    try {
      const { url } = await createBillingPortalSession();
      window.location.href = url;
    } catch (err: any) {
      alert(err.message || "Failed to open billing portal");
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="w-6 h-6 border-2 border-rho-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  const paidTiers = PRICING_TIERS.filter(
    (t) => t.id !== "free" && t.id !== "api"
  );

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">Billing</h1>
        <p className="mt-1 text-sm text-neutral-500">
          Manage your subscription and payment method
        </p>
      </div>

      {/* Current plan */}
      <div className="p-6 rounded-xl border border-neutral-800 bg-neutral-900/50 mb-8">
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-sm font-medium text-neutral-400 mb-1">
              Current Plan
            </h2>
            <p className="text-xl font-bold">
              {sub ? sub.plan_name : "Free"}
            </p>
            {sub && (
              <p className="text-sm text-neutral-500 mt-1">
                {sub.status === "active"
                  ? `Renews ${new Date(sub.current_period_end * 1000).toLocaleDateString()}`
                  : sub.status === "trialing"
                    ? `Trial ends ${new Date(sub.current_period_end * 1000).toLocaleDateString()}`
                    : `Status: ${sub.status}`}
              </p>
            )}
          </div>

          {sub && (
            <button
              onClick={handleManageBilling}
              className="flex items-center gap-2 px-4 py-2 text-sm rounded-lg border border-neutral-700 hover:border-neutral-600 text-neutral-300 hover:text-neutral-100 transition-colors"
            >
              <CreditCard className="w-4 h-4" />
              Manage Billing
            </button>
          )}
        </div>

        {/* Usage bar */}
        <div className="mt-6 pt-6 border-t border-neutral-800">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-neutral-400">Tasks this month</span>
            <span className="text-sm text-neutral-300">
              {sub?.tasks_used ?? 0} / {sub?.tasks_limit ?? 50}
            </span>
          </div>
          <div className="w-full h-2 rounded-full bg-neutral-800">
            <div
              className={cn(
                "h-2 rounded-full transition-all",
                (sub?.tasks_used ?? 0) / (sub?.tasks_limit ?? 50) > 0.8
                  ? "bg-red-500"
                  : "bg-rho-500"
              )}
              style={{
                width: `${Math.min(
                  ((sub?.tasks_used ?? 0) / (sub?.tasks_limit ?? 50)) * 100,
                  100
                )}%`,
              }}
            />
          </div>
        </div>
      </div>

      {/* Suggested plan banner */}
      {suggestedPlan && !sub && (
        <div className="p-4 rounded-xl border border-rho-500/30 bg-rho-950/20 mb-8 flex items-center gap-3">
          <AlertCircle className="w-5 h-5 text-rho-400 shrink-0" />
          <p className="text-sm text-neutral-300">
            You selected the{" "}
            <strong className="text-rho-300">
              {suggestedPlan.charAt(0).toUpperCase() + suggestedPlan.slice(1)}
            </strong>{" "}
            plan. Click below to start your 14-day free trial.
          </p>
        </div>
      )}

      {/* Upgrade options */}
      {!sub && (
        <div>
          <h2 className="text-lg font-semibold mb-4">Upgrade Your Plan</h2>
          <div className="grid md:grid-cols-2 gap-4">
            {paidTiers.map((tier) => (
              <div
                key={tier.id}
                className={cn(
                  "p-6 rounded-xl border transition-colors",
                  suggestedPlan === tier.id
                    ? "border-rho-500/50 bg-rho-950/20"
                    : "border-neutral-800 bg-neutral-900/50 hover:border-neutral-700"
                )}
              >
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <h3 className="font-semibold">{tier.name}</h3>
                    <p className="text-sm text-neutral-500">
                      {tier.description}
                    </p>
                  </div>
                  <div className="text-right">
                    <span className="text-2xl font-bold">
                      ${tier.monthlyPrice}
                    </span>
                    <span className="text-sm text-neutral-500"> / mo</span>
                  </div>
                </div>

                <ul className="space-y-2 mb-6">
                  {tier.features.slice(0, 4).map((f) => (
                    <li key={f} className="flex items-center gap-2 text-sm text-neutral-400">
                      <Check className="w-3.5 h-3.5 text-rho-400 shrink-0" />
                      {f}
                    </li>
                  ))}
                </ul>

                <button
                  onClick={() => handleCheckout(tier.id)}
                  disabled={checkoutLoading === tier.id}
                  className={cn(
                    "w-full flex items-center justify-center gap-2 py-2.5 text-sm font-medium rounded-lg transition-colors",
                    suggestedPlan === tier.id
                      ? "bg-rho-600 hover:bg-rho-700 text-white"
                      : "bg-neutral-800 hover:bg-neutral-700 text-neutral-200"
                  )}
                >
                  {checkoutLoading === tier.id ? (
                    <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                  ) : (
                    <>
                      Start Free Trial
                      <ArrowRight className="w-3.5 h-3.5" />
                    </>
                  )}
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Manage existing subscription */}
      {sub && (
        <div>
          <h2 className="text-lg font-semibold mb-4">Payment History</h2>
          <div className="p-8 rounded-xl border border-neutral-800 bg-neutral-900/50 text-center">
            <CreditCard className="w-8 h-8 text-neutral-600 mx-auto mb-3" />
            <p className="text-sm text-neutral-500">
              View invoices and update payment methods in the{" "}
              <button
                onClick={handleManageBilling}
                className="text-rho-400 hover:text-rho-300 transition-colors underline"
              >
                billing portal
              </button>
              .
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
