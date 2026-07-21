"use client";

import Link from "next/link";
import { SlidersHorizontal } from "lucide-react";
import { usePathname } from "next/navigation";

export default function ConsoleLauncher() {
  const pathname = usePathname();
  if (pathname === "/console") return null;

  return (
    <Link
      href="/console"
      title="Open Starfire's enhanced controls console"
      style={{
        position: "fixed",
        right: "16px",
        bottom: "16px",
        zIndex: 30,
        display: "inline-flex",
        alignItems: "center",
        gap: "7px",
        padding: "9px 11px",
        border: "1px solid rgba(6,182,212,.35)",
        borderRadius: "10px",
        background: "rgba(17,17,20,.94)",
        color: "#06b6d4",
        boxShadow: "0 12px 35px rgba(0,0,0,.45)",
        backdropFilter: "blur(12px)",
        fontSize: "11px",
        textDecoration: "none",
      }}
    >
      <SlidersHorizontal size={14} />
      controls
    </Link>
  );
}
