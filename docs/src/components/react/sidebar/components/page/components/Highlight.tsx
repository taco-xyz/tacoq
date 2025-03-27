// React Imports
import { RefObject, useCallback, useEffect, useMemo, useState } from "react";

// Tailwind Imports
import clsx from "clsx";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

// Types Imports
import type { Page } from "@/types/page/Page";

export function Highlight({
  title,
  children,
  parentElementRef,
}: {
  title: string;
  children: Page[];
  parentElementRef: RefObject<HTMLDivElement | null>;
}) {
  const { breadcrumbs, isPageExpanded, visiblePagesTitles } = usePageTree();

  // Find which child is in the breadcrumb path
  const focusedChildIndex = useMemo(
    () =>
      children.findIndex((child) =>
        breadcrumbs.some(
          (page) => page.metadata.title === child.metadata.title,
        ),
      ),
    [children, breadcrumbs],
  );

  // State to store the highlight position
  const [highlightPosition, setHighlightPosition] = useState<number | null>(
    null,
  );

  // Updates the highlight position
  const updateHighlightPosition = useCallback(() => {
    if (focusedChildIndex !== -1 && parentElementRef.current) {
      const childElements = parentElementRef.current.querySelectorAll(
        `[data-child-of="${title}"]`,
      );
      const focusedElement = childElements[focusedChildIndex];
      if (focusedElement) {
        const parentRect = parentElementRef.current.getBoundingClientRect();
        const childRect = focusedElement.getBoundingClientRect();
        setHighlightPosition(childRect.top - parentRect.top);
      }
    }
  }, [focusedChildIndex, parentElementRef, title]);

  useEffect(() => {
    const startTime = Date.now();
    const interval = setInterval(() => {
      updateHighlightPosition();
      if (Date.now() - startTime >= 300) {
        clearInterval(interval);
      }
    }, 16); // ~60fps

    return () => clearInterval(interval);
  }, [updateHighlightPosition, visiblePagesTitles]);

  return (
    <div
      className={clsx(
        "absolute left-[14px] h-7 w-[0.5px] rounded-full bg-zinc-800 shadow-xs shadow-zinc-900/25 transition-all duration-150 ease-in-out dark:bg-white dark:shadow-white/15",
        focusedChildIndex !== -1 && highlightPosition !== null
          ? "opacity-100"
          : "opacity-0",
      )}
      style={{
        top:
          isPageExpanded(title) && highlightPosition !== null
            ? `${highlightPosition}px`
            : "32px",
      }}
    />
  );
}
