"use client";

// React Imports
import { FC } from "react";

// Next Imports
import Link from "next/link";

// Lucide Icons
import { ChevronRight } from "lucide-react";

// Utils Imports
import clsx from "clsx";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

export const Breadcrumbs: FC = () => {
  // Extract the breadcrumbs from the page tree context
  const { breadcrumbs, isFolder } = usePageTree();

  // If there are no breadcrumbs, render the home breadcrumb
  if (!breadcrumbs.length)
    return (
      <nav className="flex w-full flex-wrap items-center gap-y-3 px-8 py-4">
        <div className="flex flex-row items-center">
          <Link
            href={"/"}
            className="pointer-events-none text-xs font-medium whitespace-nowrap text-zinc-800 transition-colors duration-150 ease-in-out dark:text-white"
          >
            Home
          </Link>
        </div>
      </nav>
    );

  return (
    <nav className="flex w-full flex-wrap items-center gap-y-3 px-8 py-4">
      {breadcrumbs
        // Filter out null values
        .filter((crumb) => crumb !== null)
        .map((crumb, index) => (
          <div key={index} className="flex flex-row items-center">
            <Link
              href={isFolder(crumb) ? "#" : crumb.url}
              className={clsx(
                "text-xs whitespace-nowrap transition-colors duration-150 ease-in-out",
                index === breadcrumbs.length - 1
                  ? "pointer-events-none font-medium text-zinc-800 dark:text-white"
                  : "text-zinc-500 dark:text-zinc-500",
                !isFolder(crumb)
                  ? "cursor-pointer hover:text-zinc-700 dark:hover:text-zinc-300"
                  : "cursor-default",
              )}
            >
              {crumb.metadata.title}
            </Link>
            {index < breadcrumbs.length - 1 && (
              <ChevronRight className="mx-2 size-3 flex-shrink-0 text-zinc-400 transition-colors duration-150 ease-in-out dark:text-zinc-600" />
            )}
          </div>
        ))}
    </nav>
  );
};
