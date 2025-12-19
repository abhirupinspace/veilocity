import type { Metadata } from "next";
import { IBM_Plex_Sans, IBM_Plex_Mono } from "next/font/google";
import "./globals.css";
import { Providers } from "@/components/providers";
import { Nav } from "@/components/nav";
import { NetworkStatus } from "@/components/network-status";
import { Toaster } from "sonner";

const ibmPlexSans = IBM_Plex_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  variable: "--font-sans",
});

const ibmPlexMono = IBM_Plex_Mono({
  subsets: ["latin"],
  weight: ["400", "500"],
  variable: "--font-mono",
});

export const metadata: Metadata = {
  title: "Veilocity Demo",
  description: "Private execution layer on Mantle",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body
        className={`${ibmPlexSans.variable} ${ibmPlexMono.variable} font-sans`}
      >
        <Providers>
          {/* Fixed grid background */}
          <div className="grid-bg fixed inset-0 opacity-30" aria-hidden="true" />

          {/* Navigation */}
          <Nav />

          {/* Main content area */}
          <div className="relative z-10 md:ml-16">
            {/* Content with top padding for header */}
            <main className="pt-24 md:pt-20 pb-16 px-4 md:px-8 min-h-screen">
              <div className="max-w-4xl mx-auto">
                <NetworkStatus />
                {children}
              </div>
            </main>
          </div>

          <Toaster
            theme="dark"
            position="bottom-right"
            toastOptions={{
              style: {
                background: "oklch(0.12 0 0)",
                border: "1px solid oklch(0.25 0 0)",
                color: "oklch(0.95 0 0)",
                fontFamily: "var(--font-mono)",
                fontSize: "12px",
              },
            }}
          />
        </Providers>
        <div className="noise-overlay" />
      </body>
    </html>
  );
}
