"use client";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";
import { usePageNavigation } from "@/components/react/sidebar/context/PageNavigationContext";
import { usePlatform } from "@/contexts/PlatformContext";

// Components Imports
import Tooltip from "@/components/react/sidebar/components/Tooltip";
import PageComponent from "@/components/react/sidebar/components/page/DesktopPage";
// import AnchorComponent from "@/app/components/sidebar/components/Anchor";

// Utils Imports
import clsx from "clsx";

// We add this prop to allow for easy layout editing through the main layout file
interface DesktopSideBarProps {
  className?: string;
}

export default function DesktopSideBar({ className }: DesktopSideBarProps) {
  // Extract the page tree context
  const { pages } = usePageTree();

  // Extract the page navigation context
  const {
    focusedPageTitle,
    startKeyboardFocus,
    endKeyboardFocus,
    pageContainerRef,
    sidebarContainerRef,
  } = usePageNavigation();

  // Extract the platform context
  const { isMacOS } = usePlatform();

  return (
    <div className="relative h-full w-full">
      <div
        ref={sidebarContainerRef}
        className={clsx(
          "custom-scrollbar flex h-full w-full flex-col gap-y-12 overflow-y-auto pr-2.5",
          className,
        )}
      >
        {/* <nav className="flex flex-col gap-y-3.5">
          {anchors.map((anchor) => (
            <AnchorComponent key={anchor.title} {...anchor} />
          ))}
        </nav> */}

        <div ref={pageContainerRef} className="flex flex-col gap-y-3">
          <div className="relative h-7">
            <button
              onClick={startKeyboardFocus}
              tabIndex={-1}
              className={`absolute left-0 cursor-pointer rounded-md bg-zinc-100/80 px-2 py-1 font-mono text-xs font-semibold whitespace-nowrap text-zinc-500 ring-1 ring-zinc-200 transition-all duration-100 ease-in-out ring-inset hover:bg-zinc-100 hover:text-zinc-700 hover:ring-zinc-300 dark:bg-zinc-900/80 dark:text-white/70 dark:ring-white/10 dark:hover:bg-zinc-900 dark:hover:text-white/90 dark:hover:ring-white/15 ${
                focusedPageTitle
                  ? "pointer-events-none opacity-0"
                  : "pointer-events-auto opacity-100"
              }`}
            >
              {isMacOS ? "⌘" : "Ctrl"} 0
            </button>

            <button
              onClick={endKeyboardFocus}
              tabIndex={-1}
              className={`absolute left-0 cursor-pointer rounded-md bg-zinc-100/80 px-2 py-1 font-mono text-xs font-semibold whitespace-nowrap text-zinc-500 ring-1 ring-zinc-200 transition-all duration-100 ease-in-out ring-inset hover:bg-zinc-100 hover:text-zinc-700 hover:ring-zinc-300 dark:bg-zinc-900/80 dark:text-white/70 dark:ring-white/10 dark:hover:bg-zinc-900 dark:hover:text-white/90 dark:hover:ring-white/15 ${
                !focusedPageTitle
                  ? "pointer-events-none opacity-0"
                  : "pointer-events-auto opacity-100"
              }`}
            >
              Esc
            </button>
          </div>

          <nav
            className="relative flex flex-col gap-y-1.5 outline-hidden"
            // The sidebar is focusable and navigable through default tab navigation
            tabIndex={0}
            onFocus={() => startKeyboardFocus()}
            onBlur={(e) => {
              // End keyboard focus when tab navigation ends
              if (!e.currentTarget.contains(e.relatedTarget as Node)) {
                console.log("Tab navigation ended");
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
      <div className="pointer-events-none absolute top-0 right-0 left-0 h-8 bg-gradient-to-b from-white to-transparent transition-[--tw-gradient-from] duration-150 ease-in-out dark:from-zinc-950" />
      {/* Bottom gradient overlay */}
      <div className="pointer-events-none absolute right-0 bottom-0 left-0 h-8 bg-gradient-to-t from-white to-transparent transition-[--tw-gradient-from] duration-150 ease-in-out dark:from-zinc-950" />
    </div>
  );
}
