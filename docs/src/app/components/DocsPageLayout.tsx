"use client";

// React Imports
import { useMemo } from "react";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

export interface DocsPageLayoutProps {
  children: React.ReactNode;
}

export default function DocsPageLayout({ children }: DocsPageLayoutProps) {
  const { currentPageTitle, getPageByTitle } = usePageTree();

  const currentPage = useMemo(() => {
    if (currentPageTitle) {
      return getPageByTitle(currentPageTitle);
    }
    return null;
  }, [getPageByTitle, currentPageTitle]);

  if (!currentPage)
    return <div className="flex flex-col gap-y-4 w-full">{children}</div>;

  return (
    <div className="flex flex-col gap-y-6 w-full flex-1">
      {/* Header */}
      <div className="flex flex-col items-start justify-start gap-y-3">
        <div className="flex flex-col items-start justify-start gap-y-1.5">
          {currentPage.metadata.badge &&
            (currentPage.metadata.badge.Icon ||
              currentPage.metadata.badge.text) && (
              <div className="flex flex-row items-center w-fit justify-center gap-x-2 text-sm font-semibold text-zinc-500 dark:text-zinc-400 transition-colors duration-150 ease-in-out">
                {currentPage.metadata.badge.Icon && (
                  <currentPage.metadata.badge.Icon className="size-3.5" />
                )}
                {currentPage.metadata.badge.text}
              </div>
            )}
          <h1 className="text-4xl font-bold tracking-tight dark:text-white text-zinc-700 transition-colors duration-150 ease-in-out">
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
    </div>
  );
}
