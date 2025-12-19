"use client";

import { usePrivy, useWallets } from "@privy-io/react-auth";
import { useAccount, useBalance } from "wagmi";
import { shortenAddress, formatEther } from "@/lib/utils";
import { LogOut, ChevronDown } from "lucide-react";
import { useState, useRef, useEffect } from "react";

export function ConnectButton() {
  const { login, logout, authenticated, ready } = usePrivy();
  const { wallets } = useWallets();
  const { address, isConnected } = useAccount();
  const { data: balance } = useBalance({ address });
  const [showDropdown, setShowDropdown] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  if (!ready) {
    return (
      <button
        disabled
        className="flex items-center gap-2 px-4 py-2 border border-border/50 font-mono text-[10px] uppercase tracking-widest text-muted-foreground"
      >
        <div className="w-3 h-3 border border-muted-foreground border-t-transparent animate-spin" />
        Loading
      </button>
    );
  }

  if (!authenticated || !isConnected) {
    return (
      <button
        onClick={login}
        className="flex items-center gap-2 px-4 py-2 border border-foreground/20 font-mono text-[10px] uppercase tracking-widest hover:border-accent hover:text-accent transition-colors"
      >
        Connect
      </button>
    );
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setShowDropdown(!showDropdown)}
        className="flex items-center gap-2 px-4 py-2 border border-border/50 font-mono text-[10px] uppercase tracking-widest hover:border-border transition-colors"
      >
        <div className="w-1.5 h-1.5 bg-green-500" />
        <span>{shortenAddress(address || "")}</span>
        <ChevronDown className={`w-3 h-3 transition-transform ${showDropdown ? "rotate-180" : ""}`} />
      </button>

      {showDropdown && (
        <div className="absolute right-0 top-full mt-2 w-56 bg-card border border-border/50 p-4 z-50">
          <div className="space-y-4">
            <div>
              <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground/60">
                Address
              </span>
              <p className="mt-1 font-mono text-xs break-all">{address}</p>
            </div>
            <div>
              <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground/60">
                Balance
              </span>
              <p className="mt-1 font-mono text-xs">
                {balance ? `${formatEther(balance.value)} ${balance.symbol}` : "—"}
              </p>
            </div>
            <div className="pt-3 border-t border-border/30">
              <button
                onClick={() => {
                  logout();
                  setShowDropdown(false);
                }}
                className="flex items-center gap-2 font-mono text-[10px] uppercase tracking-widest text-muted-foreground hover:text-foreground transition-colors"
              >
                <LogOut className="w-3 h-3" />
                Disconnect
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
