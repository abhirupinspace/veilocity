import type { Config } from "tailwindcss";

export default {
  content: [
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./lib/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "oklch(0.08 0 0)",
        foreground: "oklch(0.95 0 0)",
        card: "oklch(0.12 0 0)",
        "card-foreground": "oklch(0.95 0 0)",
        primary: "oklch(0.95 0 0)",
        "primary-foreground": "oklch(0.08 0 0)",
        secondary: "oklch(0.18 0 0)",
        "secondary-foreground": "oklch(0.85 0 0)",
        muted: "oklch(0.25 0 0)",
        "muted-foreground": "oklch(0.55 0 0)",
        accent: "oklch(0.7 0.2 45)",
        "accent-foreground": "oklch(0.08 0 0)",
        border: "oklch(0.25 0 0)",
        input: "oklch(0.2 0 0)",
        ring: "oklch(0.7 0.2 45)",
      },
      fontFamily: {
        sans: ["IBM Plex Sans", "system-ui", "sans-serif"],
        mono: ["IBM Plex Mono", "monospace"],
        display: ["Bebas Neue", "sans-serif"],
      },
    },
  },
  plugins: [],
} satisfies Config;
