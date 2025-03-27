"use client";

// React Imports
import React, { useRef } from "react";

// Next Imports
import Link from "next/link";

// Lucide Icons
import { ChevronRight } from "lucide-react";

// Tailwind Imports
import clsx from "clsx";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";
import { useMobileSidebarModal } from "../../context/MobileSidebarModalContext";

// Types Imports
import type { Page } from "@/types/page/Page";

// Components Imports
import Highlight from "./components/Highlight";

export interface PageComponentProps extends Page {
  childOf: string;
}

export default function MobilePageComponent({
  childOf,
  url,
  children,
  metadata: { title, sidebar },
}: PageComponentProps) {
  // Extract the page tree context
  const { currentPageTitle, isPageExpanded, expandPage, collapsePage } =
    usePageTree();

  // Extract the MobileSidebarContext
  const { closeSidebar } = useMobileSidebarModal();

  // Ref for the current item
  const elementRef = useRef<HTMLDivElement>(null);

  return (
    <div
      data-child-of={childOf}
      ref={elementRef}
      className={clsx("relative flex flex-col text-sm outline-hidden")}
    >
      {/* If the item has a url, it's a link */}
      {url ? (
        <div>
          <Link
            href={url}
            className={clsx(
              currentPageTitle === title &&
                "bg-zinc-800/[0.075] font-semibold text-zinc-800 hover:bg-zinc-800/[0.075] dark:bg-white/[0.075] dark:text-white dark:hover:bg-white/[0.075]",
              currentPageTitle !== title &&
                "font-normal text-zinc-600 hover:bg-zinc-800/5 hover:text-zinc-800 dark:text-zinc-300 dark:hover:bg-white/5 dark:hover:text-white",
              "relative flex w-full cursor-pointer flex-row items-center gap-2 rounded-md px-2 py-1 whitespace-nowrap outline-hidden transition-all duration-50 ease-in-out select-none",
            )}
            onClick={() => closeSidebar()}
            tabIndex={-1}
          >
            {sidebar?.Icon && (
              <sidebar.Icon
                className={clsx(
                  "mr-1 size-4 flex-shrink-0 transition-all duration-50 ease-in-out",
                  currentPageTitle === title
                    ? "text-zinc-950 dark:text-white/100"
                    : "text-zinc-500 dark:text-white/50",
                )}
              />
            )}
            {sidebar?.title ?? title}
            {children && (
              <ChevronRight
                onClick={(e) => {
                  // Prevent this form triggering the navigation to a new page
                  e.preventDefault();
                  e.stopPropagation();
                  if (isPageExpanded(title)) {
                    collapsePage(title);
                  } else {
                    expandPage(title);
                  }
                }}
                className={clsx(
                  "mt-[3px] size-3 text-zinc-700 opacity-50 transition-all duration-150 ease-in-out dark:text-zinc-300",
                  isPageExpanded(title) && "rotate-90",
                )}
              />
            )}
          </Link>
        </div>
      ) : (
        // If the item doesn't have a url, it doesn't contain a page
        <div
          onClick={() => {
            if (children) {
              if (!isPageExpanded(title)) {
                expandPage(title);
              } else {
                collapsePage(title);
              }
            }
          }}
          className={clsx(
            currentPageTitle === title &&
              "bg-zinc-800/[0.075] font-semibold text-zinc-800 hover:bg-zinc-800/[0.075] dark:bg-white/[0.075] dark:text-white dark:hover:bg-white/[0.075]",
            currentPageTitle !== title &&
              "font-normal text-zinc-600 hover:bg-zinc-800/5 hover:text-zinc-800 dark:text-zinc-300 dark:hover:bg-white/5 dark:hover:text-white",
            "relative flex w-full cursor-pointer flex-row items-center gap-2 rounded-md px-2 py-1 whitespace-nowrap outline-hidden transition-all duration-50 ease-in-out select-none",
          )}
        >
          {sidebar?.Icon && (
            <sidebar.Icon
              className={clsx(
                "mr-1 size-4 flex-shrink-0 transition-all duration-50 ease-in-out",
                currentPageTitle === title
                  ? "text-zinc-950 dark:text-white/100"
                  : "text-zinc-500 dark:text-white/50",
              )}
            />
          )}
          {sidebar?.title ?? title}
          {children && (
            <ChevronRight
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                if (isPageExpanded(title)) {
                  collapsePage(title);
                } else {
                  expandPage(title);
                }
              }}
              className={clsx(
                "mt-[3px] size-3 text-zinc-700 opacity-50 transition-all duration-150 ease-in-out dark:text-zinc-300",
                isPageExpanded(title) && "rotate-90",
              )}
            />
          )}
        </div>
      )}

      {children && (
        <div
          className={clsx(
            "ml-3.5 border-l-[1px] border-zinc-300 pl-2.5 dark:border-zinc-700",
            "grid transition-all duration-300 ease-in-out",
            isPageExpanded(title)
              ? "mt-1.5 grid-rows-[1fr] opacity-100"
              : "grid-rows-[0fr] opacity-0",
          )}
        >
          {/* Selected highlight */}
          <Highlight title={title} parentElementRef={elementRef}>
            {children}
          </Highlight>

          {/* Child items */}
          <div className="flex flex-col gap-y-2 overflow-hidden">
            {children.map((child) => (
              <MobilePageComponent
                key={child.metadata.title}
                {...child}
                childOf={title}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
