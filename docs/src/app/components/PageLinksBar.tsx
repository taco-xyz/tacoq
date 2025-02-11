"use client"

// React Imports
import { useMemo, useState, useEffect, useCallback } from "react";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

// Types Imports
import { HeadingTypes, getHeaderId } from "@/types/page/Heading";

// Utils Imports
import clsx from "clsx";

// Heroicons Imports
import { Bars3CenterLeftIcon } from "@heroicons/react/24/outline";

// Next Imports
import { useRouter } from "next/navigation";

/**

 * Renders a nested table of contents for the current page
 * Scrolls to the corresponding heading when clicked
 * Highlights the current section based on scroll position
 */
export default function PageLinksBar() {
  const router = useRouter();

  // Extract the page tree context
  const { currentPageTitle, getPageByTitle } = usePageTree();

  // Memoize the current page
  const currentPage = useMemo(() => {
    if (!currentPageTitle) return null;
    return getPageByTitle(currentPageTitle);
  }, [currentPageTitle, getPageByTitle]);

  // Set the active heading to the first heading in the current page
  const [activeHeadingId, setActiveHeadingId] = useState<string | null>(null);

  // Clicking on a heading title will scroll to it
  const handleClick = useCallback(
    (id: string) => {
      // Get the document element
      const element = document.getElementById(id);

      if (element) {
        // Scroll to the element
        element.scrollIntoView({ behavior: "smooth" });

        // Update URL without triggering a scroll
        router.push(`#${id}`, { scroll: false });
      }
    },
    [router]
  );

  // Effect to track the active heading based on the distance to the top of the viewport (94px offset)
  // Runs once on mount
  useEffect(() => {
    if (!currentPage?.content) return;

    // Check URL hash on initial load
    const hash = window.location.hash.slice(1); // Remove the # symbol

    // If the url has a hash, set the active heading to the element with the same id
    // Scroll smoothly to the element
    if (hash) {
      const element = document.getElementById(hash);
      if (element) {
        setActiveHeadingId(hash);
        element.scrollIntoView({ behavior: "smooth" });
      }
    } else {
      // If there's no hash, no scroll will be forced and the user will simply remain in the same position as before
      // We want to check what's the closest heading to the 94px mark to set it as the active heading
      let closestHeading = null;
      let closestDistance = Infinity;

      currentPage.content.forEach((heading) => {
        const element = document.getElementById(getHeaderId(heading));
        if (element) {
          // Calculate the distance between the heading and the 94px mark
          const distance = Math.abs(element.getBoundingClientRect().top - 94);
          if (distance < closestDistance) {
            closestDistance = distance;
            closestHeading = element.id;
          }
        }
      });
      if (closestHeading) {
        setActiveHeadingId(closestHeading);
      }
    }

    // Set up intersection observer for scroll updates
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setActiveHeadingId(entry.target.id);
        }
      },
      {
        rootMargin: "-94px 0px -80% 0px",
        threshold: [0.5],
      }
    );

    // Start observing all heading elements
    currentPage.content.forEach((heading) => {
      const element = document.getElementById(getHeaderId(heading));
      if (element) observer.observe(element);
    });

    return () => observer.disconnect();
  }, [currentPage?.content]);

  if (!currentPage?.content) return null;

  return (
    <nav className="flex flex-col gap-y-2 text-sm w-full">
      <span className="font-semibold text-zinc-500 dark:text-zinc-400 flex flex-row items-center gap-x-2">
        <Bars3CenterLeftIcon className="w-4 h-4" />
        On this page
      </span>

      <div className="flex flex-col gap-y-2">
        {currentPage.content.map((heading, index) => {
          const headingId = getHeaderId(heading);
          const isActive = headingId === activeHeadingId;
          return (
            <button
              key={index}
              onClick={() => handleClick(headingId)}
              className={clsx(
                "text-left hover:text-zinc-700 dark:hover:text-white transition-all rounded-sm duration-150 ease-in-out whitespace-nowrap cursor-pointer custom-tab-outline-offset-2",

                heading.type === HeadingTypes.H1 && "pl-0",
                heading.type === HeadingTypes.H2 && "pl-4",
                heading.type === HeadingTypes.H3 && "pl-8",
                isActive
                  ? "font-medium text-zinc-700 dark:text-white"
                  : "font-normal text-zinc-500 dark:text-zinc-300"
              )}
            >
              {heading.name}
            </button>
          );
        })}
      </div>
    </nav>
  );
}
