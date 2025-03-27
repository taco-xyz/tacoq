"use client";

// React Imports
import { useMemo, PropsWithChildren } from "react";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";
import Link from "next/link";

// Lucide Icons
import { ChevronLeft, ChevronRight } from "lucide-react";

export function DocsPageLayout({ children }: PropsWithChildren) {
  const { breadcrumbs, previousPage, nextPage, parentPageTitle } =
    usePageTree();

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
          {parentPageTitle ? (
            <div className="flex w-fit flex-row items-center justify-center gap-x-2 font-mono text-xs font-semibold text-zinc-500 uppercase transition-colors duration-150 ease-in-out dark:text-zinc-400">
              {parentPageTitle}
            </div>
          ) : (
            // Default to the name of the parent page if no badge info was provided
            breadcrumbs[breadcrumbs.length - 2] && (
              <div className="flex w-fit flex-row items-center justify-center gap-x-2 font-mono text-sm font-semibold text-zinc-500 transition-colors duration-150 ease-in-out dark:text-zinc-400">
                <span className="text-sm font-semibold text-zinc-500 transition-colors duration-150 ease-in-out dark:text-zinc-400">
                  {breadcrumbs[breadcrumbs.length - 2].metadata.title}
                </span>
              </div>
            )
          )}
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

      <div className="mt-8 flex w-full flex-col items-center justify-between gap-x-4 border-t border-zinc-200 transition-colors duration-150 ease-in-out sm:flex-row dark:border-zinc-800">
        {previousPage?.url && (
          <Link
            href={previousPage.url}
            className="group custom-tab-outline-offset-0 relative w-full rounded-2xl p-6 transition-all duration-150 ease-in-out sm:w-fit"
          >
            <ChevronLeft className="absolute top-1/2 left-2 size-4 -translate-y-1/2 text-zinc-400 transition-all duration-150 ease-in-out group-hover:left-1" />
            <div className="pl-6">
              <div className="text-sm text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-400">
                Previous
              </div>
              <div className="text-lg font-semibold text-zinc-900 transition-colors duration-150 ease-in-out dark:text-zinc-100">
                {previousPage.metadata.title}
              </div>
            </div>
          </Link>
        )}

        <div className="flex w-full flex-1 justify-end">
          {nextPage?.url && (
            <Link
              href={nextPage.url}
              className="group custom-tab-outline-offset-0 relative w-full rounded-2xl p-6 transition-all duration-150 ease-in-out sm:w-fit"
            >
              <ChevronRight className="absolute top-1/2 right-2 size-4 -translate-y-1/2 text-zinc-400 transition-all duration-150 ease-in-out group-hover:right-1" />
              <div className="pr-6 text-right">
                <div className="text-sm text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-400">
                  Next
                </div>
                <div className="text-lg font-semibold text-zinc-900 transition-colors duration-150 ease-in-out dark:text-zinc-100">
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
