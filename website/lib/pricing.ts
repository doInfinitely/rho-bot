/**
 * Two-tier subscription pricing for rho-bot.
 * Free (25 tasks/mo) + Pro ($12/mo unlimited).
 */

export const FREE_TASKS = 25;
export const PRO_PRICE = 12;

export const FREE_FEATURES = [
  "25 tasks per month",
  "Voice commands (TTS & STT)",
  "Unlimited sessions",
  "API access",
  "Community support",
];

export const PRO_FEATURES = [
  "Unlimited tasks",
  "Voice commands (TTS & STT)",
  "Unlimited sessions",
  "API access",
  "Priority support",
];

export const FAQ_ITEMS = [
  {
    question: "What counts as a task?",
    answer:
      "A task is a single goal-directed sequence — for example, 'book a flight to Paris' or 'fill out this spreadsheet row.' Multi-step tasks that involve many clicks and keystrokes still count as one task.",
  },
  {
    question: "What happens when I hit 25 tasks?",
    answer:
      "You'll be prompted to upgrade to Pro. Your session history and settings are preserved — upgrading just unlocks unlimited tasks instantly.",
  },
  {
    question: "How does rho-bot compare to Anthropic's computer use?",
    answer:
      "Anthropic's computer use runs on their frontier LLMs, billing you per token for every screenshot and reasoning step. A single task can cost $0.50–$3.00+. rho-bot uses distilled, purpose-built models that are dramatically cheaper while being optimized specifically for desktop automation.",
  },
  {
    question: "Can I cancel anytime?",
    answer:
      "Yes. You can cancel your Pro subscription at any time from your billing dashboard. You'll keep Pro access until the end of your billing period, then revert to the Free plan.",
  },
  {
    question: "Is there a free trial for Pro?",
    answer:
      "The Free plan is your trial — 25 tasks per month with full functionality including voice. Upgrade to Pro when you need more.",
  },
];
