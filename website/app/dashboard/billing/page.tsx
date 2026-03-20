"use client";

import { useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import { CreditCard, ArrowRight, Heart } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  getSubscription,
  createCheckoutSession,
  createBillingPortalSession,
} from "@/lib/api";
import type { Subscription } from "@/lib/api";

export default function BillingPage() {
  const searchParams = useSearchParams();

  const [sub, setSub] = useState<Subscription | null>(null);
  const [loading, setLoading] = useState(true);
  const [amount, setAmount] = useState(5);
  const [checkoutLoading, setCheckoutLoading] = useState(false);

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

  async function handleCheckout() {
    if (amount <= 0) return;
    setCheckoutLoading(true);
    try {
      const { url } = await createCheckoutSession(amount);
      window.location.href = url;
    } catch (err: any) {
      alert(err.message || "Failed to create checkout session");
    } finally {
      setCheckoutLoading(false);
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

  const isSupporter = sub && sub.amount > 0 && sub.status === "active";

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">Billing</h1>
        <p className="mt-1 text-sm text-neutral-500">
          Pay what you want — every bit helps us keep building
        </p>
      </div>

      {/* Current status */}
      <div className="p-6 rounded-xl border border-neutral-800 bg-neutral-900/50 mb-8">
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-sm font-medium text-neutral-400 mb-1">
              Current Plan
            </h2>
            <p className="text-xl font-bold">
              {isSupporter
                ? `Supporter — $${(sub.amount / 100).toFixed(0)}/mo`
                : "Free"}
            </p>
            {sub && sub.current_period_end && isSupporter && (
              <p className="text-sm text-neutral-500 mt-1">
                {sub.status === "active"
                  ? `Renews ${new Date(sub.current_period_end * 1000).toLocaleDateString()}`
                  : `Status: ${sub.status}`}
              </p>
            )}
          </div>

          {isSupporter && (
            <button
              onClick={handleManageBilling}
              className="flex items-center gap-2 px-4 py-2 text-sm rounded-lg border border-neutral-700 hover:border-neutral-600 text-neutral-300 hover:text-neutral-100 transition-colors"
            >
              <CreditCard className="w-4 h-4" />
              Manage Billing
            </button>
          )}
        </div>
      </div>

      {/* Pay what you want */}
      {!isSupporter && (
        <div className="p-8 rounded-xl border border-neutral-800 bg-neutral-900/50">
          <div className="flex items-center gap-2 mb-4">
            <Heart className="w-5 h-5 text-rho-400" />
            <h2 className="text-lg font-semibold">Support rho-bot</h2>
          </div>
          <p className="text-sm text-neutral-400 mb-6">
            rho-bot is free to use. If you find it valuable, a monthly
            contribution helps us keep improving it.
          </p>

          <div className="flex items-center gap-4 mb-6">
            <div className="flex items-baseline gap-1">
              <span className="text-lg text-neutral-400">$</span>
              <input
                type="number"
                min={1}
                step={1}
                value={amount}
                onChange={(e) => setAmount(Math.max(1, Number(e.target.value)))}
                className="w-20 text-3xl font-bold text-neutral-100 bg-transparent border-b-2 border-neutral-700 focus:border-rho-500 outline-none text-center appearance-none [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
              />
              <span className="text-sm text-neutral-500">/ month</span>
            </div>

            <div className="flex gap-2">
              {[5, 10, 25].map((v) => (
                <button
                  key={v}
                  onClick={() => setAmount(v)}
                  className={cn(
                    "px-3 py-1.5 text-sm rounded-lg transition-colors",
                    amount === v
                      ? "bg-rho-600 text-white"
                      : "bg-neutral-800 text-neutral-400 hover:bg-neutral-700"
                  )}
                >
                  ${v}
                </button>
              ))}
            </div>
          </div>

          <button
            onClick={handleCheckout}
            disabled={checkoutLoading || amount <= 0}
            className="w-full flex items-center justify-center gap-2 py-3 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 text-white transition-colors disabled:opacity-50"
          >
            {checkoutLoading ? (
              <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
            ) : (
              <>
                Support with ${amount}/mo
                <ArrowRight className="w-3.5 h-3.5" />
              </>
            )}
          </button>
        </div>
      )}

      {/* Manage existing subscription */}
      {isSupporter && (
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
