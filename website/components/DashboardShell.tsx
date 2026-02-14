"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";

const navItems = [
  { href: "/dashboard", label: "Overview" },
  { href: "/dashboard/billing", label: "Billing" },
  { href: "/dashboard/settings", label: "Settings" },
];

export default function DashboardShell({
  children,
}: {
  children: React.ReactNode;
}) {
  const pathname = usePathname();
  const router = useRouter();
  const [email, setEmail] = useState("");

  useEffect(() => {
    const token = localStorage.getItem("rho_token");
    if (!token) {
      router.push("/login");
      return;
    }
    // Decode JWT to get email (simple base64 decode of payload)
    try {
      const payload = JSON.parse(atob(token.split(".")[1]));
      setEmail(payload.email || "");
    } catch {
      // ignore
    }
  }, [router]);

  const handleLogout = () => {
    localStorage.removeItem("rho_token");
    router.push("/login");
  };

  return (
    <div className="min-h-screen flex">
      {/* Sidebar */}
      <aside className="w-56 border-r border-neutral-800 bg-neutral-950 flex flex-col">
        <div className="p-4 border-b border-neutral-800">
          <Link href="/" className="text-lg font-bold tracking-tight">
            <span className="text-rho-400">rho</span>-bot
          </Link>
        </div>

        <nav className="flex-1 p-3 space-y-1">
          {navItems.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "block px-3 py-2 text-sm rounded-lg transition-colors",
                pathname === item.href
                  ? "bg-neutral-800 text-neutral-100"
                  : "text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900"
              )}
            >
              {item.label}
            </Link>
          ))}
        </nav>

        <div className="p-4 border-t border-neutral-800">
          <p className="text-xs text-neutral-500 truncate mb-2">{email}</p>
          <button
            onClick={handleLogout}
            className="text-xs text-neutral-500 hover:text-neutral-300 transition-colors"
          >
            Log out
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 p-8 overflow-auto">{children}</main>
    </div>
  );
}
