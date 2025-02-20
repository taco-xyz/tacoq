"use client";

// React Imports
import { createPortal } from "react-dom";

// Next Imports
import dynamic from "next/dynamic";

// Tailwind Imports
import clsx from "clsx";

// Context Imports
import { useMobileSidebarModal } from "./context/MobileSidebarModalContext";
import { usePageTree } from "@/contexts/PageTreeContext";

// Components Imports
import ThemeToggle from "../topbar/components/ThemeToggle";
import GithubLink from "../topbar/components/GitHubLink";
import AnchorComponent from "./components/Anchor";
import MobilePageComponent from "./components/page/MobilePage";

// Dynamic Components Imports
const SidebarPortal = dynamic<{ children: React.ReactNode }>(
  () =>
    Promise.resolve(({ children }: { children: React.ReactNode }) =>
      createPortal(children, document.body)
    ),
  { ssr: false }
);

export default function MobileSidebarPortal() {
  // Extract the Mobile Sidebar Modal Context
  const { isSidebarOpen, closeSidebar, dialogRef } = useMobileSidebarModal();

  // Extract the Page Tree Context
  const { anchors, pages } = usePageTree();

  return (
    <SidebarPortal>
      <div
        className={clsx(
          "fixed inset-0 z-50",
          isSidebarOpen ? "pointer-events-auto" : "pointer-events-none"
        )}
      >
        {/* Backdrop */}
        <div
          className={clsx(
            "fixed inset-0 h-screen w-full dark:bg-zinc-950/20 bg-zinc-950/5 backdrop-blur-xs transition-opacity duration-300 ease-in-out",
            isSidebarOpen ? "opacity-100" : "opacity-0"
          )}
        />

        {/* Dialog */}
        <div
          ref={dialogRef}
          className={clsx(
            "dark:bg-zinc-900 bg-white fixed inset-0 left-0 w-full max-w-[300px] transition-all duration-300 ease-in-out max-h-full h-full overflow-y-auto shadow-xl dark:shadow-zinc-950/30 shadow-zinc-500/10 ring-1 ring-zinc-200 dark:ring-white/15",
            isSidebarOpen ? "translate-x-0" : "-translate-x-full"
          )}
        >
          {/* Header */}
          <div className="sticky top-0  border-b dark:border-white/10 border-zinc-200 dark:bg-zinc-900/50 bg-white/50 backdrop-blur-md overflow-hidden transition-all duration-300 ease-in-out">
            <div className="w-full h-full relative flex flex-row items-center justify-between px-8 py-5">
              {/* Theme Toggle and GitHub Link */}
              <div className="flex items-center gap-x-6">
                <GithubLink />
                <ThemeToggle />
              </div>
              {/* Close Button */}
              <button
                onClick={closeSidebar}
                className=" dark:text-white/70 dark:hover:text-white/90 cursor-pointer text-zinc-500 hover:text-zinc-700 font-semibold text-xs bg-zinc-100/80 hover:bg-zinc-100 dark:bg-zinc-900/80 dark:hover:bg-zinc-900 ring-1 ring-zinc-200 hover:ring-zinc-300 dark:ring-white/10 dark:hover:ring-white/15 transition-all duration-150 ease-in-out px-2 py-1 rounded-md whitespace-nowrap"
              >
                Close
              </button>
              {/* Decorative Background gradient */}
              <div className="absolute overflow-hidden h-full mx-auto left-1/2 -translate-x-1/2 -bottom-6 z-[-1]">
                <div className="bg-radial origin-center h-20 w-[20rem] opacity-5 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
              </div>
            </div>
          </div>

          {/* Sidebar Content */}
          <div className="px-8 py-8">
            <div className="w-full h-fit gap-y-10 flex flex-col">
              {/* Anchors */}
              <nav className="flex flex-col gap-y-3.5">
                {anchors.map((anchor) => (
                  <AnchorComponent key={anchor.title} {...anchor} />
                ))}
              </nav>

              <div className="flex flex-col gap-y-3 -ml-2">
                <nav className="flex flex-col gap-y-1.5 relative outline-hidden">
                  {/* Pages */}
                  {pages.map((page) => (
                    <MobilePageComponent
                      key={page.metadata.title}
                      childOf="root"
                      {...page}
                    />
                  ))}
                </nav>
              </div>
            </div>
          </div>
        </div>
      </div>
    </SidebarPortal>
  );
}
