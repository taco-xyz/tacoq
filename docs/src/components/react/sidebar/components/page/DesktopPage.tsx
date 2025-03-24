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
import { usePageNavigation } from "@/components/react/sidebar/context/PageNavigationContext";

// Types Imports
import type { Page } from "@/types/page/Page";

// Components Imports
import Highlight from "./components/Highlight";

export interface PageComponentProps extends Page {
  childOf: string;
}

export default function DesktopPageComponent({
  childOf,
  url,
  children,
  metadata: { title, sidebar },
}: PageComponentProps) {
  // Extract the page tree context
  const { currentPageTitle, isPageExpanded, expandPage, collapsePage } =
    usePageTree();

  // Extract the page navigation context
  const {
    focusedPageTitle,
    startHoverFocus,
    endHoverFocus,
    currentFocusedPageRef,
  } = usePageNavigation();

  // Ref for the current item
  const elementRef = useRef<HTMLDivElement>(null);

  return (
    <div
      data-child-of={childOf}
      ref={elementRef}
      className={clsx(
        "flex flex-col text-sm relative outline-hidden overflow-hidden"
      )}
    >
      {/* If the item has a url, it's a link */}
      {url ? (
        <div
          ref={focusedPageTitle === title ? currentFocusedPageRef : undefined}
        >
          <Link
            onMouseEnter={() => startHoverFocus(title)}
            onMouseLeave={() => endHoverFocus()}
            href={url}
            className={clsx(
              focusedPageTitle === title &&
                currentPageTitle === title &&
                "dark:bg-white/[0.075] bg-zinc-800/[0.075] text-zinc-800 dark:text-white font-semibold dark:hover:bg-white/[0.075] hover:bg-zinc-800/[0.075]",
              focusedPageTitle === title &&
                currentPageTitle !== title &&
                "dark:bg-white/5 bg-zinc-800/5 text-zinc-800 dark:text-white dark:hover:bg-white/5 hover:bg-zinc-800/5",
              focusedPageTitle !== title &&
                currentPageTitle === title &&
                "text-zinc-800 dark:text-white font-semibold bg-zinc-800/5 dark:bg-white/5 dark:hover:bg-white/[0.075] hover:bg-zinc-800/[0.075]",
              focusedPageTitle !== title &&
                currentPageTitle !== title &&
                "hover:text-zinc-800 dark:hover:text-white text-zinc-600 dark:text-zinc-300 font-normal dark:hover:bg-white/5 hover:bg-zinc-800/5",
              "flex items-center relative rounded-md flex-row gap-2 px-2 py-1 cursor-pointer outline-hidden select-none w-full whitespace-nowrap"
            )}
            tabIndex={-1}
          >
            {sidebar?.Icon && (
              <sidebar.Icon
                className={clsx(
                  "size-4 mr-1  transition-all duration-50 ease-in-out flex-shrink-0",
                  currentPageTitle === title
                    ? "text-zinc-950 dark:text-white/100"
                    : "text-zinc-500 dark:text-white/50"
                )}
              />
            )}
            {sidebar?.title ?? title}
            {children && (
              <ChevronRight
                onClick={(e) => {
                  // Prevent this form triggering the link navigation
                  e.preventDefault();
                  e.stopPropagation();
                  if (isPageExpanded(title)) {
                    collapsePage(title);
                  } else {
                    expandPage(title);
                  }
                }}
                className={clsx(
                  "size-3 mt-[3px] transition-all duration-150 ease-in-out opacity-50 text-zinc-700 dark:text-zinc-300",
                  isPageExpanded(title) && "rotate-90"
                )}
              />
            )}
          </Link>
        </div>
      ) : (
        // If the item doesn't have a url, it doesn't contain a page
        <div
          ref={focusedPageTitle === title ? currentFocusedPageRef : undefined}
          onMouseEnter={() => startHoverFocus(title)}
          onMouseLeave={() => endHoverFocus()}
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
            focusedPageTitle === title &&
              currentPageTitle === title &&
              "dark:bg-white/[0.075] bg-zinc-800/[0.075] text-zinc-800 dark:text-white font-semibold dark:hover:bg-white/[0.075] hover:bg-zinc-800/[0.075]",
            focusedPageTitle === title &&
              currentPageTitle !== title &&
              "dark:bg-white/5 bg-zinc-800/5 text-zinc-800 dark:text-white dark:hover:bg-white/5 hover:bg-zinc-800/5",
            focusedPageTitle !== title &&
              currentPageTitle === title &&
              "text-zinc-800 dark:text-white font-semibold bg-zinc-800/5 dark:bg-white/5 dark:hover:bg-white/[0.075] hover:bg-zinc-800/[0.075]",
            focusedPageTitle !== title &&
              currentPageTitle !== title &&
              "hover:text-zinc-800 dark:hover:text-white text-zinc-600 dark:text-zinc-300 font-normal dark:hover:bg-white/5 hover:bg-zinc-800/5",
            "flex items-center relative rounded-md flex-row gap-2 px-2 py-1 cursor-pointer outline-hidden select-none w-full whitespace-nowrap"
          )}
        >
          {sidebar?.Icon && (
            <sidebar.Icon
              className={clsx(
                "size-4 mr-1  transition-all duration-50 ease-in-out flex-shrink-0",
                currentPageTitle === title
                  ? "text-zinc-950 dark:text-white/100"
                  : "text-zinc-500 dark:text-white/50"
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
                "size-3 mt-[3px] transition-all duration-150 ease-in-out opacity-50 text-zinc-700 dark:text-zinc-300",
                isPageExpanded(title) && "rotate-90"
              )}
            />
          )}
        </div>
      )}

      {children && (
        <div
          className={clsx(
            "pl-2.5 ml-3.5 border-l-[1px] border-zinc-300 dark:border-zinc-700",
            "grid transition-all duration-300 ease-in-out",
            isPageExpanded(title)
              ? "grid-rows-[1fr] opacity-100 mt-1.5"
              : "grid-rows-[0fr] opacity-0"
          )}
        >
          {/* Selected highlight */}
          <Highlight title={title} parentElementRef={elementRef}>
            {children}
          </Highlight>

          {/* Child items */}
          <div className="flex flex-col gap-y-2 overflow-hidden">
            {children.map((child) => (
              <DesktopPageComponent
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
