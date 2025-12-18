# Veilocity Demo

A minimal Next.js demo for the Veilocity private execution layer on Mantle.

## Features

- **Real-time Stats**: Live TVL, deposit count, and state root from the VeilocityVault contract
- **Privy Wallet**: Easy wallet connection with Privy (supports external wallets + email login)
- **Private Deposits**: Deposit MNT with commitment generation
- **Live Event Feed**: Watch deposits happen in real-time

## Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Create a `.env.local` file:
   ```bash
   cp .env.local.example .env.local
   ```

3. Get a Privy App ID from [dashboard.privy.io](https://dashboard.privy.io) and add it to `.env.local`:
   ```
   NEXT_PUBLIC_PRIVY_APP_ID=your-privy-app-id
   ```

4. Configure the contract address in `lib/abi.ts`:
   ```typescript
   export const VAULT_ADDRESSES: Record<number, `0x${string}`> = {
     // Mantle Sepolia Testnet
     5003: "0xYOUR_DEPLOYED_CONTRACT_ADDRESS",
   };
   ```

5. Run the development server:
   ```bash
   npm run dev
   ```

6. Open [http://localhost:3000](http://localhost:3000)

## Configuration

### Contract Address

Update the vault address in `lib/abi.ts` after deploying the VeilocityVault contract:

```typescript
export const VAULT_ADDRESSES: Record<number, `0x${string}`> = {
  5003: "0x...", // Mantle Sepolia
  5000: "0x...", // Mantle Mainnet (optional)
};
```

### Network

The demo is configured for Mantle Sepolia by default. To use mainnet, update `lib/wagmi.ts` and the providers.

## Tech Stack

- Next.js 15
- Privy for wallet connection
- wagmi + viem for contract interactions
- Tailwind CSS with OKLCH colors (matching Veilocity branding)
- Sonner for toast notifications

## Important Notes

- **Save your secret!** When making a deposit, a random 32-byte secret is generated. You must save this to withdraw later.
- The demo uses `keccak256` for commitment generation. The production Veilocity system uses Poseidon hash.
- This is for testnet use only. Do not use real funds.
