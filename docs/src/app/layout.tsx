// Global Styles Imports
import "./globals.css";

// Components Imports
import { TopBar } from "../components/react/topbar/TopBar";
import { Footer } from "../components/react/Footer";
import DesktopSideBar from "../components/react/sidebar/DesktopSideBar";
import DocsPageLayout from "../components/react/docs-page-layout/DocsPageLayout";
import PageLinksBar from "../components/react/PageLinksBar";

// Context imports
import { TooltipProvider } from "@/components/react/sidebar/context/TooltipContext";
import { PageTreeProvider } from "@/contexts/PageTreeContext";
import { PageNavigationProvider } from "@/components/react/sidebar/context/PageNavigationContext";
import { PlatformProvider } from "@/contexts/PlatformContext";

// Fonts Imports
import { Geist, Geist_Mono } from "next/font/google";

// Utils Imports
import clsx from "clsx";

// Fonts
const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

// Metadata
export const metadata = {
  metadataBase: new URL("https://www.tacodivision.com"),
  title: {
    template: "%s | TacoQ Documentation",
    default: "TacoQ | Multi-Language Distributed Task Queue",
  },
  description:
    "TacoQ is a multi-language distributed task queue with built-in observability, low latency, and first-class idiomatic support for Python, Rust, and JavaScript.",
  keywords: [
    "tacoq",
    "task queue",
    "distributed system",
    "message broker",
    "rabbitmq",
    "postgres",
    "python",
    "rust",
    "javascript",
    "async tasks",
    "worker queue",
    "job queue",
    "observability",
    "multi-language",
    "distributed computing",
  ],
  authors: [{ name: "Taco Division" }],
  openGraph: {
    type: "website",
    locale: "en_US",
    siteName: "TacoQ Docs",
    title: "TacoQ | Multi-Language Distributed Task Queue",
    description:
      "Modern distributed task queue with multi-language support, built-in observability, and low latency.",
  },
  twitter: {
    card: "summary_large_image",
    title: "TacoQ Documentation",
    description: "Modern distributed task queue with multi-language support",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html
      lang="en"
      className={clsx(
        geistSans.variable,
        geistMono.variable,
        "dark custom-body-scrollbar",
      )}
      suppressHydrationWarning
    >
      <head>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              document.documentElement.classList.toggle(
                "dark",
                localStorage.theme === "dark" ||
                (!("theme" in localStorage) &&
                  window.matchMedia("(prefers-color-scheme: dark)").matches)
              );
            `,
          }}
        />
      </head>
      <body className="relative flex min-h-screen w-full flex-col items-center overflow-x-hidden bg-white text-zinc-800 transition-colors duration-150 ease-in-out dark:bg-zinc-950 dark:text-white">
        <PageTreeProvider>
          <PlatformProvider>
            {/* Topbar */}
            <div className="sticky top-0 z-10 w-full">
              <TopBar />
            </div>
            <div className="relative flex w-full max-w-(--breakpoint-2xl) flex-row items-start justify-between gap-x-10 px-8 md:py-8 xl:gap-x-16 2xl:gap-x-20">
              {/* Sidebar - height is calculated to account for the topbar and bottom padding */}
              <div className="sticky top-[112px] z-1 hidden h-[calc(100vh-112px-32px)] w-56 flex-col md:flex xl:w-64">
                <PageNavigationProvider>
                  <TooltipProvider>
                    <DesktopSideBar className="py-6" />
                  </TooltipProvider>
                </PageNavigationProvider>
              </div>
              {/* Page */}
              <div className="z-0 w-full min-w-0 flex-1 py-6">
                <DocsPageLayout>{children}</DocsPageLayout>
              </div>
              {/* Page Links Bar - height is calculated to account for the topbar and bottom padding */}
              <div className="sticky top-[112px] z-1 hidden h-[calc(100vh-112px-32px)] w-56 flex-col lg:flex xl:w-64">
                <PageLinksBar className="py-6" />
              </div>
            </div>
            {/* Footer */}
            <div className="absolute top-full w-full">
              <Footer />
            </div>
          </PlatformProvider>
        </PageTreeProvider>
      </body>
    </html>
  );
}
