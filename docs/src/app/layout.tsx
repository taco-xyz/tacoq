// Global Styles Imports
import "./globals.css";

// Components Imports
import { TopBar } from "./components/topbar/TopBar";
import { Footer } from "./components/Footer";
import SideBar from "./components/sidebar/SideBar";
import DocsPageLayout from "./components/DocsPageLayout";
import PageLinksBar from "./components/PageLinksBar";

// Context imports
import { TooltipProvider } from "@/app/components/sidebar/context/TooltipContext";
import { PageTreeProvider } from "@/contexts/PageTreeContext";
import { PageNavigationProvider } from "@/app/components/sidebar/context/PageNavigationContext";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="">
      <body className="flex pt-[76px] pb-[68px] items-center text-zinc-700 dark:text-white relative min-h-screen w-full flex-col overflow-x-hidden bg-white transition-colors duration-150 ease-in-out dark:bg-zinc-950">
        <div className="fixed top-0 w-full z-10">
          <TopBar />
        </div>

        <div className="flex flex-row items-start justify-between gap-x-20 w-full max-w-(--breakpoint-2xl) relative py-8 px-8">
          <PageTreeProvider>
            <div className="flex h-[700px] flex-col w-64 sticky top-[108px] z-1">
              <PageNavigationProvider>
                <TooltipProvider>
                  <SideBar />
                </TooltipProvider>
              </PageNavigationProvider>
            </div>
            <div className="z-0 w-full flex-1">
              <DocsPageLayout>{children}</DocsPageLayout>
            </div>
            <div className="flex h-[500px] flex-col w-64 sticky top-[108px] z-1">
              <PageLinksBar />
            </div>
          </PageTreeProvider>
        </div>
        <Footer />
      </body>
    </html>
  );
}
