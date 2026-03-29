"use client";

import { useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import { CreditCard, ArrowRight, Zap } from "lucide-react";
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
    setCheckoutLoading(true);
    try {
      const { url } = await createCheckoutSession();
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

  const isPro = sub && sub.plan_id === "pro" && sub.status === "active";
  const tasksUsed = sub?.tasks_used ?? 0;
  const tasksLimit = sub?.tasks_limit ?? 25;
  const isUnlimited = tasksLimit >= 999_999_999;
  const usagePercent = isUnlimited ? 0 : Math.min(100, (tasksUsed / tasksLimit) * 100);

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">Billing</h1>
        <p className="mt-1 text-sm text-neutral-500">
          Manage your subscription and track usage
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
              {isPro ? "Pro — Unlimited" : "Free — 25 tasks/mo"}
            </p>
            {sub && sub.current_period_end && isPro && (
              <p className="text-sm text-neutral-500 mt-1">
                {sub.status === "active"
                  ? `Renews ${new Date(sub.current_period_end * 1000).toLocaleDateString()}`
                  : `Status: ${sub.status}`}
              </p>
            )}
          </div>

          {isPro && (
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

      {/* Usage */}
      <div className="p-6 rounded-xl border border-neutral-800 bg-neutral-900/50 mb-8">
        <h2 className="text-sm font-medium text-neutral-400 mb-3">
          Tasks This Month
        </h2>
        <div className="flex items-baseline gap-2 mb-3">
          <span className="text-3xl font-bold">{tasksUsed}</span>
          <span className="text-sm text-neutral-500">
            / {isUnlimited ? "Unlimited" : tasksLimit}
          </span>
        </div>
        {!isUnlimited && (
          <div className="w-full h-2 rounded-full bg-neutral-800 overflow-hidden">
            <div
              className={`h-full rounded-full transition-all ${
                usagePercent >= 90
                  ? "bg-red-500"
                  : usagePercent >= 70
                    ? "bg-yellow-500"
                    : "bg-rho-500"
              }`}
              style={{ width: `${usagePercent}%` }}
            />
          </div>
        )}
      </div>

      {/* Upgrade CTA for free users */}
      {!isPro && (
        <div className="p-8 rounded-xl border border-rho-500/30 bg-rho-950/20">
          <div className="flex items-center gap-2 mb-4">
            <Zap className="w-5 h-5 text-rho-400" />
            <h2 className="text-lg font-semibold">Upgrade to Pro</h2>
          </div>
          <p className="text-sm text-neutral-400 mb-6">
            Get unlimited tasks, priority support, and never hit a quota wall.
          </p>

          <button
            onClick={handleCheckout}
            disabled={checkoutLoading}
            className="w-full flex items-center justify-center gap-2 py-3 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 text-white transition-colors disabled:opacity-50"
          >
            {checkoutLoading ? (
              <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
            ) : (
              <>
                Upgrade to Pro — $12/mo
                <ArrowRight className="w-3.5 h-3.5" />
              </>
            )}
          </button>
        </div>
      )}

      {/* Payment history for Pro users */}
      {isPro && (
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
