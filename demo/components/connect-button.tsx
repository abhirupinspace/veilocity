"use client";

import { usePrivy, useWallets } from "@privy-io/react-auth";
import { useAccount, useBalance } from "wagmi";
import { shortenAddress, formatEther } from "@/lib/utils";
import { Wallet, LogOut, ChevronDown } from "lucide-react";
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
        className="flex items-center gap-2 px-4 py-2 bg-secondary text-muted-foreground font-mono text-sm"
      >
        <div className="w-4 h-4 border-2 border-muted-foreground border-t-transparent rounded-full animate-spin" />
        Loading...
      </button>
    );
  }

  if (!authenticated || !isConnected) {
    return (
      <button
        onClick={login}
        className="flex items-center gap-2 px-4 py-2 bg-accent text-accent-foreground font-mono text-sm hover:opacity-90 transition-opacity"
      >
        <Wallet className="w-4 h-4" />
        Connect Wallet
      </button>
    );
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setShowDropdown(!showDropdown)}
        className="flex items-center gap-2 px-4 py-2 bg-secondary border border-border font-mono text-sm hover:bg-secondary/80 transition-colors"
      >
        <div className="w-2 h-2 bg-green-500 rounded-full" />
        <span>{shortenAddress(address || "")}</span>
        <ChevronDown className={`w-4 h-4 transition-transform ${showDropdown ? "rotate-180" : ""}`} />
      </button>

      {showDropdown && (
        <div className="absolute right-0 top-full mt-2 w-64 bg-card border border-border p-4 z-50">
          <div className="space-y-3">
            <div>
              <p className="text-xs text-muted-foreground font-mono uppercase tracking-wider">Address</p>
              <p className="font-mono text-sm break-all">{address}</p>
            </div>
            <div>
              <p className="text-xs text-muted-foreground font-mono uppercase tracking-wider">Balance</p>
              <p className="font-mono text-sm">
                {balance ? `${formatEther(balance.value)} ${balance.symbol}` : "..."}
              </p>
            </div>
            <div className="border-t border-border pt-3">
              <button
                onClick={() => {
                  logout();
                  setShowDropdown(false);
                }}
                className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors w-full"
              >
                <LogOut className="w-4 h-4" />
                Disconnect
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
