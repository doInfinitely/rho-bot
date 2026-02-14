"use client";

import Link from "next/link";
import { useState } from "react";

export default function Navbar() {
  const [open, setOpen] = useState(false);

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 border-b border-neutral-800/60 bg-neutral-950/80 backdrop-blur-xl">
      <div className="mx-auto max-w-6xl flex items-center justify-between px-6 h-16">
        <Link href="/" className="flex items-center gap-2">
          <span className="text-xl font-bold tracking-tight">
            <span className="text-rho-400">rho</span>-bot
          </span>
        </Link>

        {/* Desktop links */}
        <div className="hidden md:flex items-center gap-8">
          <a href="/#how-it-works" className="text-sm text-neutral-400 hover:text-neutral-100 transition-colors">
            How It Works
          </a>
          <Link href="/pricing" className="text-sm text-neutral-400 hover:text-neutral-100 transition-colors">
            Pricing
          </Link>
          <Link
            href="/login"
            className="text-sm text-neutral-400 hover:text-neutral-100 transition-colors"
          >
            Log In
          </Link>
          <Link
            href="/signup"
            className="px-4 py-2 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 transition-colors"
          >
            Get Started
          </Link>
        </div>

        {/* Mobile toggle */}
        <button
          onClick={() => setOpen(!open)}
          className="md:hidden text-neutral-400"
        >
          <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            {open ? (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            ) : (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            )}
          </svg>
        </button>
      </div>

      {/* Mobile menu */}
      {open && (
        <div className="md:hidden border-t border-neutral-800 px-6 py-4 space-y-3 bg-neutral-950">
          <a href="/#how-it-works" className="block text-sm text-neutral-400">How It Works</a>
          <Link href="/pricing" className="block text-sm text-neutral-400">Pricing</Link>
          <Link href="/login" className="block text-sm text-neutral-400">Log In</Link>
          <Link href="/signup" className="block text-sm text-rho-400 font-medium">Get Started</Link>
        </div>
      )}
    </nav>
  );
}
