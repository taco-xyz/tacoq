"use client";

// React Imports
import React, { useRef } from "react";

// Next Imports
import Link from "next/link";

// Heroicons Imports
import { ChevronRightIcon } from "@heroicons/react/24/outline";

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
      className={clsx("flex flex-col text-sm relative outline-hidden")}
    >
      {/* If the item has a url, it's a link */}
      {url ? (
        <div>
          <Link
            href={url}
            className={clsx(
              currentPageTitle === title &&
                "dark:bg-white/[0.075] bg-zinc-800/[0.075] text-zinc-700 dark:text-white font-semibold dark:hover:bg-white/[0.075] hover:bg-zinc-800/[0.075]",
              currentPageTitle !== title &&
                "hover:text-zinc-700 dark:hover:text-white text-zinc-600 dark:text-zinc-300 font-normal dark:hover:bg-white/5 hover:bg-zinc-800/5",
              "flex items-center relative rounded-md flex-row gap-2 px-2 py-1 cursor-pointer outline-hidden select-none w-full whitespace-nowrap transition-all duration-50 ease-in-out"
            )}
            onClick={() => {
              closeSidebar();
            }}
            tabIndex={-1}
          >
            {sidebar?.Icon && (
              <sidebar.Icon
                className={clsx(
                  "size-3.5 mr-1  transition-all duration-50 ease-in-out",
                  currentPageTitle === title
                    ? "text-zinc-950 dark:text-white/100"
                    : "text-zinc-500 dark:text-white/50"
                )}
              />
            )}
            {sidebar?.title ?? title}
            {children && (
              <ChevronRightIcon
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
              "dark:bg-white/[0.075] bg-zinc-800/[0.075] text-zinc-700 dark:text-white font-semibold dark:hover:bg-white/[0.075] hover:bg-zinc-800/[0.075]",
            currentPageTitle !== title &&
              "hover:text-zinc-700 dark:hover:text-white text-zinc-600 dark:text-zinc-300 font-normal dark:hover:bg-white/5 hover:bg-zinc-800/5",
            "flex items-center relative rounded-md flex-row gap-2 px-2 py-1 cursor-pointer outline-hidden select-none w-full whitespace-nowrap transition-all duration-50 ease-in-out"
          )}
        >
          {sidebar?.Icon && (
            <sidebar.Icon
              className={clsx(
                "size-3.5 mr-1  transition-all duration-50 ease-in-out",
                currentPageTitle === title
                  ? "text-zinc-950 dark:text-white/100"
                  : "text-zinc-500 dark:text-white/50"
              )}
            />
          )}
          {sidebar?.title ?? title}
          {children && (
            <ChevronRightIcon
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
