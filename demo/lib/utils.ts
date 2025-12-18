import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatEther(wei: bigint): string {
  const ether = Number(wei) / 1e18;
  if (ether < 0.001) return "< 0.001";
  if (ether < 1) return ether.toFixed(4);
  if (ether < 1000) return ether.toFixed(3);
  return ether.toLocaleString(undefined, { maximumFractionDigits: 2 });
}

export function shortenAddress(address: string): string {
  if (!address) return "";
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

export function shortenHash(hash: string): string {
  if (!hash) return "";
  return `${hash.slice(0, 10)}...${hash.slice(-8)}`;
}
