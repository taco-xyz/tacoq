"use client";

// React Imports
import { useMemo } from "react";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";
import Link from "next/link";

// Heroicons
import { ChevronLeftIcon, ChevronRightIcon } from "@heroicons/react/24/outline";

export interface DocsPageLayoutProps {
  children: React.ReactNode;
}

export default function DocsPageLayout({ children }: DocsPageLayoutProps) {
  const { breadcrumbs, previousPage, nextPage, parentPageTitle } =
    usePageTree();

  // Get the current page for simplicity
  const currentPage = useMemo(() => {
    return breadcrumbs[breadcrumbs.length - 1];
  }, [breadcrumbs]);

  // If there is no current page, render the children
  if (!currentPage)
    return <div className="flex flex-col gap-y-4 w-full">{children}</div>;

  return (
    <div className="flex flex-col gap-y-4 w-full flex-1">
      {/* Header */}
      <div className="flex flex-col items-start justify-start gap-y-5 border-b border-zinc-200 dark:border-zinc-800 pb-9 transition-colors duration-150 ease-in-out">
        <div className="flex flex-col items-start justify-start gap-y-3">
          {parentPageTitle ? (
            <div className="flex flex-row items-center w-fit justify-center gap-x-2 font-mono uppercase text-xs font-semibold text-zinc-500 dark:text-zinc-400 transition-colors duration-150 ease-in-out">
              {parentPageTitle}
            </div>
          ) : (
            // Default to the name of the parent page if no badge info was provided
            breadcrumbs[breadcrumbs.length - 2] && (
              <div className="flex flex-row items-center w-fit justify-center gap-x-2 font-mono text-sm font-semibold text-zinc-500 dark:text-zinc-400 transition-colors duration-150 ease-in-out">
                <span className="text-sm font-semibold text-zinc-500 dark:text-zinc-400 transition-colors duration-150 ease-in-out">
                  {breadcrumbs[breadcrumbs.length - 2].metadata.title}
                </span>
              </div>
            )
          )}
          <h1 className="text-4xl font-semibold tracking-tight dark:text-white text-zinc-800 transition-colors duration-150 ease-in-out">
            {currentPage.metadata.title}
          </h1>
        </div>
        {currentPage.metadata.description && (
          <h5 className="text-lg font-[450] tracking-normal dark:text-zinc-300 text-zinc-600 transition-colors duration-150 ease-in-out">
            {currentPage.metadata.description}
          </h5>
        )}
      </div>

      <div className="flex flex-col gap-y-4 w-full">
        {/* Content */}
        {children}
      </div>

      <div className="flex sm:flex-row flex-col items-center w-full justify-between gap-x-4 mt-8 border-t border-zinc-200 dark:border-zinc-800">
       
          {previousPage?.url && (
            <Link
              href={previousPage.url}
              className="w-full sm:w-fit group rounded-2xl relative p-6  custom-tab-outline-offset-2 transition-all duration-150 ease-in-out"
            >
              <ChevronLeftIcon className="size-4 absolute left-2 group-hover:left-1 top-1/2 -translate-y-1/2 text-zinc-400 transition-all duration-150 ease-in-out" />
              <div className="pl-6">
                <div className="text-sm dark:text-zinc-400 text-zinc-600">
                  Previous
                </div>
                <div className="text-lg dark:text-zinc-100 text-zinc-900 font-semibold">
                  {previousPage.metadata.title}
                </div>
              </div>
            </Link>
          )}
        
        <div className="flex-1 flex justify-end w-full">
          {nextPage?.url && (
            <Link
              href={nextPage.url}
              className="w-full sm:w-fit group rounded-2xl relative p-6  custom-tab-outline-offset-2 transition-all duration-150 ease-in-out"
            >
              <ChevronRightIcon className="size-4 absolute right-2 group-hover:right-1 top-1/2 -translate-y-1/2 text-zinc-400 transition-all duration-150 ease-in-out" />
              <div className="pr-6 text-right">
                <div className="text-sm dark:text-zinc-400 text-zinc-600">
                  Next
                </div>
                <div className="text-lg dark:text-zinc-100 text-zinc-900 font-semibold">
                  {nextPage.metadata.title}
                </div>
              </div>
            </Link>
          )}
        </div>
      </div>
    </div>
  );
}
