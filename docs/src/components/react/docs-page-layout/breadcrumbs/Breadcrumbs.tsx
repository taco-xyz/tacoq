"use client";

// Next Imports
import Link from "next/link";

// Lucide Icons
import { ChevronRight } from "lucide-react";

// Utils Imports
import clsx from "clsx";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

export default function Breadcrumbs() {
  // Extract the breadcrumbs from the page tree context
  const { breadcrumbs } = usePageTree();

  // If there are no breadcrumbs, render the home breadcrumb
  if (!breadcrumbs.length)
    return (
      <nav className="flex flex-wrap items-center gap-y-3 py-4 w-full px-8">
        <div className="flex items-center flex-row">
          <Link
            href={"/"}
            className="text-xs whitespace-nowrap transition-colors duration-150 ease-in-out text-zinc-800 dark:text-white font-medium pointer-events-none"
          >
            Home
          </Link>
        </div>
      </nav>
    );

  return (
    <nav className="flex flex-wrap items-center gap-y-3 py-4 w-full px-8">
      {breadcrumbs.map((crumb, index) => (
        <div key={crumb.metadata.title} className="flex items-center flex-row">
          <Link
            href={crumb.url ?? "#"}
            className={clsx(
              "text-xs whitespace-nowrap transition-colors duration-150 ease-in-out",
              index === breadcrumbs.length - 1
                ? "text-zinc-800 dark:text-white font-medium pointer-events-none"
                : "text-zinc-500 dark:text-zinc-500",
              crumb.url
                ? "cursor-pointer hover:text-zinc-700 dark:hover:text-zinc-300"
                : "cursor-default"
            )}
          >
            {crumb.metadata.title}
          </Link>
          {index < breadcrumbs.length - 1 && (
            <ChevronRight className="size-3 mx-2 text-zinc-400 dark:text-zinc-600 flex-shrink-0 transition-colors duration-150 ease-in-out" />
          )}
        </div>
      ))}
    </nav>
  );
}
