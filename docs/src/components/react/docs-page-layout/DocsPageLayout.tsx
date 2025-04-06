"use client";

// React Imports
import { FC, useMemo, PropsWithChildren } from "react";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";
import Link from "next/link";

// Lucide Icons
import { ChevronLeft, ChevronRight } from "lucide-react";

export const DocsPageLayout: FC<PropsWithChildren> = ({ children }) => {
  const { breadcrumbs, previousElement, nextElement, isFolder } = usePageTree();

  // Get the current page for simplicity
  const currentPage = useMemo(() => {
    return breadcrumbs[breadcrumbs.length - 1];
  }, [breadcrumbs]);

  // If there is no current page, render the children
  if (!currentPage)
    return <div className="flex w-full flex-col gap-y-4">{children}</div>;

  return (
    <div className="flex w-full flex-1 flex-col gap-y-4">
      {/* Header */}
      <div className="flex flex-col items-start justify-start gap-y-5 border-b border-zinc-200 pb-9 transition-colors duration-150 ease-in-out dark:border-zinc-800">
        <div className="flex flex-col items-start justify-start gap-y-3">
          {
            // Default to the name of the parent page if it exists
            breadcrumbs[breadcrumbs.length - 2] && (
              <div className="flex w-fit flex-row items-center justify-center gap-x-2 font-mono text-sm font-semibold text-zinc-500 transition-colors duration-150 ease-in-out dark:text-zinc-400">
                <span className="text-sm font-semibold text-zinc-500 transition-colors duration-150 ease-in-out dark:text-zinc-400">
                  {breadcrumbs[breadcrumbs.length - 2]?.metadata.title ?? ""}
                </span>
              </div>
            )
          }
          <h1 className="text-4xl font-semibold tracking-tight text-zinc-800 transition-colors duration-150 ease-in-out dark:text-white">
            {currentPage.metadata.title}
          </h1>
        </div>
        {currentPage.metadata.description && (
          <h5 className="text-lg font-[450] tracking-normal text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-300">
            {currentPage.metadata.description}
          </h5>
        )}
      </div>

      <div className="flex w-full flex-col gap-y-4">
        {/* Content */}
        {children}
      </div>

      {!isFolder(currentPage) && (
        <p className="flex w-full justify-start text-left text-xs text-zinc-400 transition-colors duration-150 ease-in-out dark:text-zinc-700">
          Last updated: {new Date(currentPage.lastUpdated).toLocaleDateString()}
        </p>
      )}

      <div className="mt-3 flex w-full flex-col items-center justify-between gap-x-4 border-t border-zinc-200 transition-colors duration-150 ease-in-out sm:flex-row dark:border-zinc-800">
        {previousElement && !isFolder(previousElement) && (
          <Link
            href={previousElement.url}
            className="group custom-tab-outline-offset-0 relative w-full rounded-2xl p-6 transition-all duration-150 ease-in-out sm:w-fit"
          >
            <ChevronLeft className="absolute top-1/2 left-2 size-4 -translate-y-1/2 text-zinc-400 transition-all duration-150 ease-in-out group-hover:left-1" />
            <div className="pl-6">
              <div className="text-sm text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-400">
                Previous
              </div>
              <div className="text-lg font-semibold text-zinc-900 transition-colors duration-150 ease-in-out dark:text-zinc-100">
                {previousElement.metadata.title}
              </div>
            </div>
          </Link>
        )}

        <div className="flex w-full flex-1 justify-end">
          {nextElement && !isFolder(nextElement) && (
            <Link
              href={nextElement.url}
              className="group custom-tab-outline-offset-0 relative w-full rounded-2xl p-6 transition-all duration-150 ease-in-out sm:w-fit"
            >
              <ChevronRight className="absolute top-1/2 right-2 size-4 -translate-y-1/2 text-zinc-400 transition-all duration-150 ease-in-out group-hover:right-1" />
              <div className="pr-6 text-right">
                <div className="text-sm text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-400">
                  Next
                </div>
                <div className="text-lg font-semibold text-zinc-900 transition-colors duration-150 ease-in-out dark:text-zinc-100">
                  {nextElement.metadata.title}
                </div>
              </div>
            </Link>
          )}
        </div>
      </div>
    </div>
  );
};
