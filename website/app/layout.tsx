import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "rho-bot | Hierarchical Goal Induction Agent",
  description:
    "An autonomous desktop agent powered by hierarchical goal induction. It observes, learns, and acts.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen flex flex-col">{children}</body>
    </html>
  );
}
