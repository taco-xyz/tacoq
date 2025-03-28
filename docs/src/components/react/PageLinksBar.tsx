"use client";

// React Imports
import { useMemo, useState, useEffect, useCallback } from "react";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

// Types Imports
import { getHeaderId } from "@/utils/getHeaderId";

// Utils Imports
import clsx from "clsx";

// Lucide Icons
import { ChartNoAxesGantt } from "lucide-react";

// Next Imports
import { useRouter } from "next/navigation";

interface PageLinksBarProps {
  className?: string;
}

/**

 * Renders a nested table of contents for the current page
 * Scrolls to the corresponding heading when clicked
 * Highlights the current section based on scroll position
 */
export function PageLinksBar({ className }: PageLinksBarProps) {
  const router = useRouter();

  // Extract the page tree context
  const { breadcrumbs } = usePageTree();

  // Memoize the current page (the last breadcrumb)
  const currentPage = useMemo(() => {
    if (!breadcrumbs.length) return null;
    return breadcrumbs[breadcrumbs.length - 1];
  }, [breadcrumbs]);

  // Set the active heading to the first heading in the current page
  const [activeHeadingId, setActiveHeadingId] = useState<string | null>(null);

  // Clicking on a heading title will scroll to it
  const handleClick = useCallback(
    (id: string) => {
      // Get the document element
      const element = document.getElementById(id);
      if (!element) return;

      // Scroll to the element
      element.scrollIntoView({ behavior: "smooth" });

      // Update URL without triggering a scroll
      router.push(`#${id}`, { scroll: false });
    },
    [router],
  );

  // Effect to track the active heading based on the distance to the top of the viewport (94px offset)
  // Runs once on mount
  useEffect(() => {
    if (!currentPage?.headers) return;

    // Check URL hash on initial load
    const hash = window.location.hash.slice(1); // Remove the # symbol

    // If the url has a hash, scroll smoothly to the element
    if (hash) {
      const element = document.getElementById(hash);
      if (element) {
        element.scrollIntoView({ behavior: "smooth" });
      }
    }

    // We want to check what's the closest heading to the 94px mark to set it as the active heading
    let closestHeading = null;
    let closestDistance = Infinity;

    currentPage.headers.forEach((header) => {
      const element = document.getElementById(getHeaderId(header));
      if (!element) return;

      // Calculate the distance between the heading and the 94px mark
      const distance = Math.abs(element.getBoundingClientRect().top - 94);
      if (distance >= closestDistance) return;

      // Update the closest heading and distance
      closestDistance = distance;
      closestHeading = element.id;
    });

    if (closestHeading) {
      setActiveHeadingId(closestHeading);
    }

    // Set up intersection observer for scroll updates
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) return;
        setActiveHeadingId(entry.target.id);
      },
      {
        // Calculate percentage that gives us exactly 40px zone below the 94px header:
        // percentage = ((viewportHeight - 94px - 40px) / viewportHeight) * 100
        rootMargin: (() => {
          const viewportHeight = window.innerHeight;
          const headerOffset = 94;
          const zoneHeight = 40;
          const percentage =
            ((viewportHeight - headerOffset - zoneHeight) / viewportHeight) *
            100;
          return `-${headerOffset}px 0px -${percentage}% 0px`;
        })(),
        threshold: [0.5],
      },
    );

    // Start observing all heading elements
    currentPage.headers.forEach((header) => {
      const element = document.getElementById(getHeaderId(header));
      if (!element) return;
      observer.observe(element);
    });

    return () => observer.disconnect();
  }, [currentPage?.headers]);

  if (!currentPage?.headers) return null;

  return (
    <div className="relative h-full w-full">
      {/* Top gradient overlay */}
      <div className="pointer-events-none absolute top-0 right-0 left-0 h-8 bg-gradient-to-b from-white to-transparent transition-[--tw-gradient-from] duration-150 ease-in-out dark:from-zinc-950" />
      <nav
        className={clsx(
          "custom-scrollbar flex h-full w-full flex-col gap-y-2 overflow-y-auto pr-2.5 pl-[1px] text-sm",
          className,
        )}
      >
        {/* Title */}
        <span className="-ml-[3px] flex flex-row items-center gap-x-2 font-semibold text-zinc-500 dark:text-zinc-400">
          <ChartNoAxesGantt className="size-4" />
          On this page
        </span>

        {/* Links */}
        <div className="flex flex-col gap-y-2">
          {currentPage.headers.map((header, index) => {
            const headingId = getHeaderId(header);
            const isActive = headingId === activeHeadingId;
            return (
              <button
                key={index}
                onClick={() => handleClick(headingId)}
                className={clsx(
                  "custom-tab-outline-offset-0 w-fit cursor-pointer rounded-md text-left transition-all duration-150 ease-in-out hover:text-zinc-800 dark:hover:text-white",

                  header.type === "h1" && "pl-0",
                  header.type === "h2" && "pl-4",
                  header.type === "h3" && "pl-8",
                  isActive
                    ? "font-medium text-zinc-800 dark:text-white"
                    : "font-normal text-zinc-500 dark:text-zinc-300",
                )}
              >
                {header.title}
              </button>
            );
          })}
        </div>
      </nav>
      {/* Bottom gradient overlay */}
      <div className="pointer-events-none absolute right-0 bottom-0 left-0 h-8 bg-gradient-to-t from-white to-transparent transition-[--tw-gradient-from] duration-150 ease-in-out dark:from-zinc-950" />
    </div>
  );
}
