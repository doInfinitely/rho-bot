/**
 * Pay-what-you-want pricing for rho-bot.
 */

export const DEFAULT_AMOUNT = 5; // dollars
export const MIN_AMOUNT = 0;

export const FEATURES = [
  "Unlimited tasks",
  "Unlimited concurrent sessions",
  "Unlimited session history",
  "Priority execution speed",
  "API access",
  "Custom task templates",
  "Community support",
];

export const FAQ_ITEMS = [
  {
    question: "What counts as a task?",
    answer:
      "A task is a single goal-directed sequence — for example, 'book a flight to Paris' or 'fill out this spreadsheet row.' Multi-step tasks that involve many clicks and keystrokes still count as one task.",
  },
  {
    question: "Can I really pay $0?",
    answer:
      "Yes. rho-bot is free to use. If you find it valuable, paying what you can helps us keep building and improving it.",
  },
  {
    question: "How does rho-bot compare to Anthropic's computer use?",
    answer:
      "Anthropic's computer use runs on their frontier LLMs, billing you per token for every screenshot and reasoning step. A single task can cost $0.50–$3.00+. rho-bot uses distilled, purpose-built models that are dramatically cheaper while being optimized specifically for desktop automation.",
  },
  {
    question: "Can I change my amount later?",
    answer:
      "Yes. You can update your contribution at any time from your billing dashboard. Changes take effect on your next billing cycle.",
  },
  {
    question: "Is there a free trial?",
    answer:
      "You don't need one — the product is free to use. Pay what you want, when you want.",
  },
];
