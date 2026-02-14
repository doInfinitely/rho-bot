/**
 * Pricing tiers and plan definitions for rho-bot.
 */

export interface PricingTier {
  id: string;
  name: string;
  description: string;
  monthlyPrice: number | null; // null = custom pricing
  annualPrice: number | null;
  taskLimit: string;
  effectiveRate: string;
  cta: string;
  ctaHref: string;
  highlighted: boolean;
  features: string[];
}

export const PRICING_TIERS: PricingTier[] = [
  {
    id: "free",
    name: "Free",
    description: "For trying things out",
    monthlyPrice: 0,
    annualPrice: 0,
    taskLimit: "50 tasks / month",
    effectiveRate: "",
    cta: "Get Started",
    ctaHref: "/signup",
    highlighted: false,
    features: [
      "50 tasks per month",
      "Standard execution speed",
      "Community support",
      "1 active agent session",
      "7-day session history",
    ],
  },
  {
    id: "pro",
    name: "Pro",
    description: "For power users",
    monthlyPrice: 25,
    annualPrice: 20,
    taskLimit: "500 tasks / month",
    effectiveRate: "~$0.05 / task",
    cta: "Start Free Trial",
    ctaHref: "/signup?plan=pro",
    highlighted: true,
    features: [
      "500 tasks per month",
      "Priority execution speed",
      "Email support",
      "3 concurrent agent sessions",
      "90-day session history",
      "API access",
      "Custom task templates",
    ],
  },
  {
    id: "team",
    name: "Team",
    description: "For collaborative teams",
    monthlyPrice: 80,
    annualPrice: 64,
    taskLimit: "2,000 tasks / seat / month",
    effectiveRate: "~$0.04 / task",
    cta: "Start Free Trial",
    ctaHref: "/signup?plan=team",
    highlighted: false,
    features: [
      "2,000 tasks per seat per month",
      "Priority execution speed",
      "Priority support with SLA",
      "10 concurrent agent sessions",
      "Unlimited session history",
      "API access",
      "Custom task templates",
      "Team workspace & sharing",
      "Role-based access control",
    ],
  },
  {
    id: "api",
    name: "API",
    description: "Pay-as-you-go for developers",
    monthlyPrice: null,
    annualPrice: null,
    taskLimit: "Unlimited",
    effectiveRate: "from $0.02 / task",
    cta: "Contact Sales",
    ctaHref: "mailto:sales@rho-bot.dev",
    highlighted: false,
    features: [
      "Unlimited tasks",
      "Per-task billing ($0.02–$0.08)",
      "Per-step billing ($0.005 / step)",
      "Dedicated support engineer",
      "Unlimited concurrent sessions",
      "Unlimited session history",
      "Full REST & WebSocket API",
      "On-premise deployment option",
      "Custom model fine-tuning",
      "99.9% uptime SLA",
    ],
  },
];

export const FAQ_ITEMS = [
  {
    question: "What counts as a task?",
    answer:
      "A task is a single goal-directed sequence — for example, 'book a flight to Paris' or 'fill out this spreadsheet row.' Multi-step tasks that involve many clicks and keystrokes still count as one task.",
  },
  {
    question: "How does rho-bot compare to Anthropic's computer use?",
    answer:
      "Anthropic's computer use runs on their frontier LLMs, billing you per token for every screenshot and reasoning step. A single task can cost $0.50–$3.00+. rho-bot uses distilled, purpose-built models that are 10–50x cheaper per task while being optimized specifically for desktop automation.",
  },
  {
    question: "Can I switch plans at any time?",
    answer:
      "Yes. Upgrades take effect immediately with prorated billing. Downgrades take effect at the next billing cycle. You can also cancel anytime — no lock-in.",
  },
  {
    question: "What happens if I exceed my task limit?",
    answer:
      "Overage tasks are billed at $0.10 per task. You'll get an alert at 80% and 100% of your limit so there are no surprises.",
  },
  {
    question: "Is the API tier separate from the subscription tiers?",
    answer:
      "Yes. The API tier is purely pay-as-you-go with no monthly fee. It's designed for developers embedding rho-bot into their own applications. If you want both a dashboard and API access, the Pro and Team plans include API access as well.",
  },
  {
    question: "Do you offer a free trial for paid plans?",
    answer:
      "Yes — Pro and Team plans come with a 14-day free trial. No credit card required to start.",
  },
];
