"use client";

// React Imports
import { createPortal } from "react-dom";

// Next Imports
import dynamic from "next/dynamic";

// Lucide Icons
import { X } from "lucide-react";

// Tailwind Imports
import clsx from "clsx";

// Context Imports
import { useMobileSidebarModal } from "./context/MobileSidebarModalContext";
import { usePageTree } from "@/contexts/PageTreeContext";

// Components Imports
import ThemeToggle from "../topbar/components/ThemeToggle";
import MobilePageComponent from "./components/page/MobilePage";
import Logo from "../Logo";
//import AnchorComponent from "./components/Anchor";

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
  const { pages } = usePageTree();

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
            "dark:bg-zinc-950 bg-white fixed right-0 top-0 bottom-0 w-full sm:max-w-sm max-w-full transition-all duration-300 ease-in-out max-h-full h-full overflow-y-auto shadow-xl dark:shadow-zinc-950/30 shadow-zinc-500/10 border-l border-zinc-200 dark:border-white/5",
            isSidebarOpen ? "translate-x-0" : "translate-x-full"
          )}
        >
          {/* Header */}
          <div className="sticky top-0 dark:bg-zinc-950/50 bg-white/50 backdrop-blur-lg overflow-hidden transition-all duration-300 ease-in-out px-8 ">
            <div className="w-full h-full relative flex flex-row items-center justify-between border-b dark:border-white/10 border-zinc-200 py-5 transition-all duration-300 ease-in-out">
              {/* Logo */}
              <Logo />

              {/* Theme Toggle and Close Button */}
              <div className="flex items-center gap-x-6">
                <ThemeToggle />
                <button
                  onClick={closeSidebar}
                  className="dark:hover:text-white/80 text-zinc-500 hover:text-zinc-400 cursor-pointer dark:text-white/70 transition-all ease-in-out duration-150 custom-tab-outline-offset-4 rounded-xs"
                >
                  <X className="size-6" />
                </button>
              </div>

              {/* Decorative Background gradient */}
              <div className="absolute overflow-hidden h-full mx-auto left-1/2 -translate-x-1/2 -bottom-6 z-[-1] pointer-events-none">
                <div className="bg-radial origin-center h-20 w-[20rem] dark:opacity-3 opacity-10 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% pointer-events-none" />
              </div>
            </div>
          </div>

          {/* Sidebar Content */}
          <div className="px-8 py-8">
            <div className="w-full h-fit gap-y-10 flex flex-col">
              {/*<nav className="flex flex-col gap-y-3.5">
                {anchors.map((anchor) => (
                  <AnchorComponent key={anchor.title} {...anchor} />
                ))}
              </nav>*/}

              <div className="flex flex-col gap-y-3 -ml-2 ">
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
