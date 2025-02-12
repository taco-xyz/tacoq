// Global Styles Imports
import "./globals.css";

// Components Imports
import { TopBar } from "./components/topbar/TopBar";
import { Footer } from "./components/Footer";
import SideBar from "./components/sidebar/SideBar";
import DocsPageLayout from "./components/DocsPageLayout";
import PageLinksBar from "./components/PageLinksBar";

// Custom Icons Imports
import { GithubIcon, XIcon, DiscordIcon } from "./components/icons/social";

// Context imports
import { TooltipProvider } from "@/app/components/sidebar/context/TooltipContext";
import { PageTreeProvider } from "@/contexts/PageTreeContext";
import { PageNavigationProvider } from "@/app/components/sidebar/context/PageNavigationContext";

const footerInfo = {
  linkGroups: [
    {
      groupName: "Product",
      links: [
        { linkName: "Features", url: "/features" },
        { linkName: "Pricing", url: "/pricing" },
        { linkName: "Documentation", url: "/docs" },
      ],
    },
    {
      groupName: "Resources",
      links: [
        { linkName: "Blog", url: "/blog" },
        { linkName: "Support", url: "/support" },
        { linkName: "API", url: "/api" },
      ],
    },
    {
      groupName: "Company",
      links: [
        { linkName: "About", url: "/about" },
        { linkName: "Careers", url: "/careers" },
        { linkName: "Contact", url: "/contact" },
      ],
    },
    {
      groupName: "Legal",
      links: [
        { linkName: "Privacy", url: "/privacy" },
        { linkName: "Terms", url: "/terms" },
        { linkName: "Security", url: "/security" },
      ],
    },
  ],
  socialLinks: [
    { Icon: GithubIcon, url: "https://github.com/your-repo" },
    { Icon: XIcon, url: "https://twitter.com/your-handle" },
    { Icon: DiscordIcon, url: "https://discord.gg/your-server" },
  ],
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
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
        <div className="absolute top-full w-full">
          <Footer footerInfo={footerInfo} />
        </div>
      </body>
    </html>
  );
}
