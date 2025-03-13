// Global Styles Imports
import "./globals.css";

// Components Imports
import { TopBar } from "./components/topbar/TopBar";
import { Footer } from "./components/Footer";
import DesktopSideBar from "./components/sidebar/DesktopSideBar";
import DocsPageLayout from "./components/docs-page-layout/DocsPageLayout";
import PageLinksBar from "./components/PageLinksBar";

// Context imports
import { TooltipProvider } from "@/app/components/sidebar/context/TooltipContext";
import { PageTreeProvider } from "@/contexts/PageTreeContext";
import { PageNavigationProvider } from "@/app/components/sidebar/context/PageNavigationContext";

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
      <body className="flex items-center text-zinc-700 dark:text-white relative min-h-screen w-full flex-col overflow-x-hidden bg-white transition-colors duration-150 ease-in-out dark:bg-zinc-950">
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
