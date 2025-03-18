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

export const metadata = {
  metadataBase: new URL("https://www.tacodivision.com/  "),
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
    images: [
      {
        url: "/TacoQBannerLight.svg",
        width: 1000,
        height: 300,
        alt: "TacoQ Banner",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "TacoQ Documentation",
    description: "Modern distributed task queue with multi-language support",
    images: ["/TacoQBannerLight.svg"],
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
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
      className={clsx(geistSans.variable, geistMono.variable, "dark")}
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
      <body className="flex items-center text-zinc-800 dark:text-white relative min-h-screen w-full flex-col overflow-x-hidden bg-white transition-colors duration-150 ease-in-out dark:bg-zinc-950">
        <PageTreeProvider>
          {/* Topbar */}
          <div className="sticky top-0 w-full z-10">
            <TopBar />
          </div>
          <div className="flex flex-row items-start justify-between 2xl:gap-x-20 xl:gap-x-16 gap-x-10 w-full max-w-(--breakpoint-2xl) relative md:py-8 px-8 ">
            {/* Sidebar - height is calculated to account for the topbar and bottom padding */}
            <div className="h-[calc(100vh-112px-32px)] flex-col xl:w-64 w-56 sticky top-[112px] z-1 md:flex hidden">
              <PageNavigationProvider>
                <TooltipProvider>
                  <DesktopSideBar className="py-6" />
                </TooltipProvider>
              </PageNavigationProvider>
            </div>
            {/* Page */}
            <div className="z-0 w-full flex-1 py-6">
              <DocsPageLayout>{children}</DocsPageLayout>
            </div>
            {/* Page Links Bar - height is calculated to account for the topbar and bottom padding */}
            <div className="h-[calc(100vh-112px-32px)] flex-col xl:w-64 w-56 sticky top-[112px] z-1 lg:flex hidden">
              <PageLinksBar className="py-6" />
            </div>
          </div>
          {/* Footer */}
          <div className="absolute top-full w-full">
            <Footer />
          </div>
        </PageTreeProvider>
      </body>
    </html>
  );
}
