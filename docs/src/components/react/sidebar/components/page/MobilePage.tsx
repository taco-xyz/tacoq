"use client";

// React Imports
import { FC, useRef, useMemo } from "react";

// Next Imports
import Link from "next/link";

// Lucide Icons
import { ChevronRight } from "lucide-react";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";
import { useMobileSidebarModal } from "../../context/MobileSidebarModalContext";

// Types Imports
import type { PageTreeElement } from "@/types/page-tree-element/PageTreeElement";

// Components Imports
import { Highlight } from "./components/Highlight";

// Utils Imports
import { getIcon } from "@/utils/getIcon";
import clsx from "clsx";

export interface PageComponentProps {
  childOf: string;
  pageTreeElement: PageTreeElement;
}

export const MobilePageComponent: FC<PageComponentProps> = ({
  childOf,
  pageTreeElement,
}) => {
  // Extract the page tree context
  const {
    currentElementTitle,
    isFolderExpanded,
    expandFolder,
    collapseFolder,
    isFolder,
  } = usePageTree();

  // Extract the MobileSidebarContext
  const { closeSidebar } = useMobileSidebarModal();

  // Ref for the current item
  const elementRef = useRef<HTMLDivElement>(null);

  // Get the current icon component
  const Icon = useMemo(() => {
    if (pageTreeElement.metadata.icon) {
      return getIcon(pageTreeElement.metadata.icon);
    }
    return null;
  }, [pageTreeElement.metadata.icon]);

  return (
    <div
      data-child-of={childOf}
      ref={elementRef}
      className={clsx("relative flex flex-col text-sm outline-hidden")}
    >
      {/* If the item has a url, it's a link */}
      {!isFolder(pageTreeElement) ? (
        <>
          {/* Page Sidebar Item */}
          <div>
            <Link
              href={pageTreeElement.url}
              className={clsx(
                currentElementTitle === pageTreeElement.metadata.title &&
                  "bg-zinc-800/[0.075] font-semibold text-zinc-800 hover:bg-zinc-800/[0.075] dark:bg-white/[0.075] dark:text-white dark:hover:bg-white/[0.075]",
                currentElementTitle !== pageTreeElement.metadata.title &&
                  "font-normal text-zinc-600 hover:bg-zinc-800/5 hover:text-zinc-800 dark:text-zinc-300 dark:hover:bg-white/5 dark:hover:text-white",
                "relative flex w-full cursor-pointer flex-row items-center gap-2 rounded-md px-2 py-1 whitespace-nowrap outline-hidden transition-all duration-50 ease-in-out select-none",
              )}
              onClick={() => closeSidebar()}
              tabIndex={-1}
            >
              {Icon && (
                <Icon
                  className={clsx(
                    "mr-1 size-4 flex-shrink-0 transition-all duration-50 ease-in-out",
                    currentElementTitle === pageTreeElement.metadata.title
                      ? "text-zinc-950 dark:text-white/100"
                      : "text-zinc-500 dark:text-white/50",
                  )}
                />
              )}
              {pageTreeElement.metadata.title}
            </Link>
          </div>
        </>
      ) : (
        <>
          {/* Folder Sidebar Item */}
          <div
            onClick={() => {
              if (!isFolderExpanded(pageTreeElement.metadata.title)) {
                expandFolder(pageTreeElement.metadata.title);
              } else {
                collapseFolder(pageTreeElement.metadata.title);
              }
            }}
            className={clsx(
              currentElementTitle === pageTreeElement.metadata.title &&
                "bg-zinc-800/[0.075] font-semibold text-zinc-800 hover:bg-zinc-800/[0.075] dark:bg-white/[0.075] dark:text-white dark:hover:bg-white/[0.075]",
              currentElementTitle !== pageTreeElement.metadata.title &&
                "font-normal text-zinc-600 hover:bg-zinc-800/5 hover:text-zinc-800 dark:text-zinc-300 dark:hover:bg-white/5 dark:hover:text-white",
              "relative flex w-full cursor-pointer flex-row items-center gap-2 rounded-md px-2 py-1 whitespace-nowrap outline-hidden transition-all duration-50 ease-in-out select-none",
            )}
          >
            {Icon && (
              <Icon
                className={clsx(
                  "mr-1 size-4 flex-shrink-0 transition-all duration-50 ease-in-out",
                  currentElementTitle === pageTreeElement.metadata.title
                    ? "text-zinc-950 dark:text-white/100"
                    : "text-zinc-500 dark:text-white/50",
                )}
              />
            )}
            {pageTreeElement.metadata.title}

            <ChevronRight
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                if (isFolderExpanded(pageTreeElement.metadata.title)) {
                  collapseFolder(pageTreeElement.metadata.title);
                } else {
                  expandFolder(pageTreeElement.metadata.title);
                }
              }}
              className={clsx(
                "mt-[3px] size-3 text-zinc-700 opacity-50 transition-all duration-150 ease-in-out dark:text-zinc-300",
                isFolderExpanded(pageTreeElement.metadata.title) && "rotate-90",
              )}
            />
          </div>

          {/* Child items */}
          <div
            className={clsx(
              "ml-3.5 border-l-[1px] border-zinc-300 pl-2.5 dark:border-zinc-700",
              "grid transition-all duration-300 ease-in-out",
              isFolderExpanded(pageTreeElement.metadata.title)
                ? "mt-1.5 grid-rows-[1fr] opacity-100"
                : "grid-rows-[0fr] opacity-0",
            )}
          >
            {/* Selected highlight */}
            <Highlight
              title={pageTreeElement.metadata.title}
              parentElementRef={elementRef}
            >
              {pageTreeElement.children}
            </Highlight>

            {/* Child items */}
            <div className="flex flex-col gap-y-2 overflow-hidden">
              {pageTreeElement.children.map((child, index) => (
                <MobilePageComponent
                  key={index}
                  pageTreeElement={child}
                  childOf={pageTreeElement.metadata.title}
                />
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
};
