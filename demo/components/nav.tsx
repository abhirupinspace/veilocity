"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ConnectButton } from "./connect-button";
import { ArrowDownLeft, ArrowUpRight, LayoutDashboard, History } from "lucide-react";

const navItems = [
  { href: "/", label: "Dashboard", icon: LayoutDashboard },
  { href: "/deposit", label: "Deposit", icon: ArrowDownLeft },
  { href: "/withdraw", label: "Withdraw", icon: ArrowUpRight },
  { href: "/history", label: "History", icon: History },
];

export function Nav() {
  const pathname = usePathname();

  return (
    <>
      {/* Side Nav - Desktop */}
      <nav className="fixed left-0 top-0 z-50 h-screen w-16 hidden md:flex flex-col justify-center border-r border-border/30 bg-background/95 backdrop-blur-sm">
        <div className="flex flex-col gap-2 px-3">
          {navItems.map(({ href, label, icon: Icon }) => {
            const isActive = pathname === href;
            return (
              <Link
                key={href}
                href={href}
                className="group relative flex items-center justify-center p-3 transition-colors"
                title={label}
              >
                <Icon
                  className={`w-4 h-4 transition-colors ${
                    isActive ? "text-accent" : "text-muted-foreground group-hover:text-foreground"
                  }`}
                />
                {isActive && (
                  <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-6 bg-accent" />
                )}
                {/* Tooltip */}
                <span className="absolute left-full ml-3 px-2 py-1 bg-card border border-border/50 font-mono text-[10px] uppercase tracking-widest opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap">
                  {label}
                </span>
              </Link>
            );
          })}
        </div>
      </nav>

      {/* Top Header - Mobile */}
      <header className="md:hidden fixed top-0 left-0 right-0 z-50 border-b border-border/30 bg-background/95 backdrop-blur-sm">
        <div className="px-4 py-3 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-2">
            <span className="font-mono text-xs uppercase tracking-[0.2em] text-accent">Veilocity</span>
          </Link>
          <ConnectButton />
        </div>
        {/* Mobile Nav */}
        <div className="px-4 pb-3 flex items-center gap-4 overflow-x-auto scrollbar-hide">
          {navItems.map(({ href, label, icon: Icon }) => {
            const isActive = pathname === href;
            return (
              <Link
                key={href}
                href={href}
                className={`flex items-center gap-2 font-mono text-[10px] uppercase tracking-widest whitespace-nowrap transition-colors ${
                  isActive ? "text-foreground" : "text-muted-foreground"
                }`}
              >
                <Icon className={`w-3 h-3 ${isActive ? "text-accent" : ""}`} />
                {label}
              </Link>
            );
          })}
        </div>
      </header>

      {/* Desktop Header */}
      <header className="hidden md:block fixed top-0 left-16 right-0 z-40 border-b border-border/30 bg-background/95 backdrop-blur-sm">
        <div className="px-8 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="font-mono text-xs uppercase tracking-[0.2em] text-accent">Veilocity</span>
            <span className="text-muted-foreground/30">|</span>
            <span className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
              {navItems.find(item => item.href === pathname)?.label || "Demo"}
            </span>
          </div>
          <ConnectButton />
        </div>
      </header>
    </>
  );
}
