"use client";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";
import { usePageNavigation } from "@/app/components/sidebar/context/PageNavigationContext";

// Components Imports
import Tooltip from "@/app/components/sidebar/components/Tooltip";
import PageComponent from "@/app/components/sidebar/components/page/DesktopPage";
import AnchorComponent from "@/app/components/sidebar/components/Anchor";

// Utils Imports
import clsx from "clsx";

// We add this prop to allow for easy layout editing through the main layout file
interface DesktopSideBarProps {
  className?: string;
}

export default function DesktopSideBar({ className }: DesktopSideBarProps) {
  // Extract the page tree context
  const { pages, anchors } = usePageTree();

  // Extract the page navigation context
  const {
    focusedPageTitle,
    startKeyboardFocus,
    endKeyboardFocus,
    pageContainerRef,
    sidebarContainerRef,
  } = usePageNavigation();

  return (
    <div className="w-full h-full relative">
      <div
        ref={sidebarContainerRef}
        className={clsx(
          "w-full h-full gap-y-12 flex flex-col overflow-y-scroll scrollbar-hidden",
          className
        )}
      >
        <nav className="flex flex-col gap-y-3.5">
          {anchors.map((anchor) => (
            <AnchorComponent key={anchor.title} {...anchor} />
          ))}
        </nav>

        <div ref={pageContainerRef} className="flex flex-col gap-y-3">
          <div className="relative h-7">
            <button
              onClick={startKeyboardFocus}
              tabIndex={-1}
              className={`absolute left-0 font-mono ring-inset dark:text-white/70 dark:hover:text-white/90 cursor-pointer text-zinc-500 hover:text-zinc-700 font-semibold text-xs bg-zinc-100/80 hover:bg-zinc-100 dark:bg-zinc-900/80 dark:hover:bg-zinc-900 ring-1 ring-zinc-200 hover:ring-zinc-300 dark:ring-white/10 dark:hover:ring-white/15 transition-all duration-100 ease-in-out px-2 py-1 rounded-md whitespace-nowrap ${
                focusedPageTitle
                  ? "opacity-0 pointer-events-none"
                  : "opacity-100 pointer-events-auto"
              }`}
            >
              Ctrl 0
            </button>

            <button
              onClick={endKeyboardFocus}
              tabIndex={-1}
              className={`absolute left-0 font-mono ring-inset dark:text-white/70 dark:hover:text-white/90 cursor-pointer text-zinc-500 hover:text-zinc-700 font-semibold text-xs bg-zinc-100/80 hover:bg-zinc-100 dark:bg-zinc-900/80 dark:hover:bg-zinc-900 ring-1 ring-zinc-200 hover:ring-zinc-300 dark:ring-white/10 dark:hover:ring-white/15 transition-all duration-100 ease-in-out px-2 py-1 rounded-md whitespace-nowrap ${
                !focusedPageTitle
                  ? "opacity-0 pointer-events-none"
                  : "opacity-100 pointer-events-auto"
              }`}
            >
              Esc
            </button>
          </div>

          <nav
            className="flex flex-col gap-y-1.5 relative outline-hidden"
            // The sidebar is focusable and navigable through default tab navigation
            tabIndex={0}
            onFocus={() => {
              startKeyboardFocus();
            }}
            onBlur={(e) => {
              // End keyboard focus only on tab navigation
              if (!e.currentTarget.contains(e.relatedTarget as Node)) {
                endKeyboardFocus();
              }
            }}
          >
            {/* Tooltip */}
            <Tooltip />

            {/* Pages */}
            {pages.map((page) => (
              <PageComponent
                key={page.metadata.title}
                childOf="root"
                {...page}
              />
            ))}
          </nav>
        </div>
      </div>
      {/* Top gradient overlay */}
      <div className="absolute top-0 left-0 right-0 h-8 bg-gradient-to-b from-white dark:from-zinc-950 to-transparent pointer-events-none transition-[--tw-gradient-from] duration-150 ease-in-out" />
      {/* Bottom gradient overlay */}
      <div className="absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-white dark:from-zinc-950 to-transparent pointer-events-none transition-[--tw-gradient-from] duration-150 ease-in-out" />
    </div>
  );
}
