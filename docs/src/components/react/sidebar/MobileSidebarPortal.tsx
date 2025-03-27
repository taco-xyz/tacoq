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
      createPortal(children, document.body),
    ),
  { ssr: false },
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
          isSidebarOpen ? "pointer-events-auto" : "pointer-events-none",
        )}
      >
        {/* Backdrop */}
        <div
          className={clsx(
            "fixed inset-0 h-screen w-full bg-zinc-950/5 backdrop-blur-xs transition-opacity duration-300 ease-in-out dark:bg-zinc-950/20",
            isSidebarOpen ? "opacity-100" : "opacity-0",
          )}
        />

        {/* Dialog */}
        <div
          ref={dialogRef}
          className={clsx(
            "fixed top-0 right-0 bottom-0 h-full max-h-full w-full max-w-full overflow-y-auto border-l border-zinc-200 bg-white shadow-xl shadow-zinc-500/10 transition-all duration-300 ease-in-out sm:max-w-sm dark:border-white/5 dark:bg-zinc-950 dark:shadow-zinc-950/30",
            isSidebarOpen ? "translate-x-0" : "translate-x-full",
          )}
        >
          {/* Header */}
          <div className="sticky top-0 overflow-hidden bg-white/50 px-8 backdrop-blur-lg transition-all duration-300 ease-in-out dark:bg-zinc-950/50">
            <div className="relative flex h-full w-full flex-row items-center justify-between border-b border-zinc-200 py-5 transition-all duration-300 ease-in-out dark:border-white/10">
              {/* Logo */}
              <Logo />

              {/* Theme Toggle and Close Button */}
              <div className="flex items-center gap-x-6">
                <ThemeToggle />
                <button
                  onClick={closeSidebar}
                  className="custom-tab-outline-offset-4 cursor-pointer rounded-xs text-zinc-500 transition-all duration-150 ease-in-out hover:text-zinc-400 dark:text-white/70 dark:hover:text-white/80"
                >
                  <X className="size-6" />
                </button>
              </div>

              {/* Decorative Background gradient */}
              <div className="pointer-events-none absolute -bottom-6 left-1/2 z-[-1] mx-auto h-full -translate-x-1/2 overflow-hidden">
                <div className="pointer-events-none h-20 w-[20rem] origin-center bg-radial from-zinc-400 from-0% via-zinc-400/50 via-15% to-transparent to-50% opacity-10 dark:from-white dark:via-white/50 dark:opacity-3" />
              </div>
            </div>
          </div>

          {/* Sidebar Content */}
          <div className="px-8 py-8">
            <div className="flex h-fit w-full flex-col gap-y-10">
              {/*<nav className="flex flex-col gap-y-3.5">
                {anchors.map((anchor) => (
                  <AnchorComponent key={anchor.title} {...anchor} />
                ))}
              </nav>*/}

              <div className="-ml-2 flex flex-col gap-y-3">
                <nav className="relative flex flex-col gap-y-1.5 outline-hidden">
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
