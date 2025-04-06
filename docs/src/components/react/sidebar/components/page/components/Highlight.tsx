// React Imports
import {
  FC,
  RefObject,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";

// Tailwind Imports
import clsx from "clsx";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

// Types Imports
import type { PageTreeElement } from "@/types/page-tree-element/PageTreeElement";

interface HighlightProps {
  title: string;
  children: PageTreeElement[];
  parentElementRef: RefObject<HTMLDivElement | null>;
}

export const Highlight: FC<HighlightProps> = ({
  title,
  children,
  parentElementRef,
}) => {
  const { breadcrumbs, isFolderExpanded, visibleElementsTitles } =
    usePageTree();

  // Find which child is in the breadcrumb path
  const focusedChildIndex = useMemo(
    () =>
      children.findIndex((child) =>
        breadcrumbs.some(
          (page) => page && page.metadata.title === child.metadata.title,
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
    // Return if the focused child index is not found or the parent element is not available
    if (focusedChildIndex === -1 || !parentElementRef.current) return;

    const childElements = parentElementRef.current.querySelectorAll(
      `[data-child-of="${title}"]`,
    );
    const focusedElement = childElements[focusedChildIndex];

    // Return if the focused element is not found
    if (!focusedElement) return;

    // Get the parent and child element bounding client rects
    const parentRect = parentElementRef.current.getBoundingClientRect();
    const childRect = focusedElement.getBoundingClientRect();

    // Set the highlight position
    setHighlightPosition(childRect.top - parentRect.top);
  }, [focusedChildIndex, parentElementRef, title]);

  // Update the highlight position every 16ms
  useEffect(() => {
    const startTime = Date.now();
    const interval = setInterval(() => {
      // Update the highlight position
      updateHighlightPosition();

      // Return if the interval has not reached 300ms
      if (Date.now() - startTime < 300) return;

      // Clear the interval
      clearInterval(interval);
    }, 16); // ~60fps

    return () => clearInterval(interval);
  }, [updateHighlightPosition, visibleElementsTitles]);

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
          isFolderExpanded(title) && highlightPosition !== null
            ? `${highlightPosition}px`
            : "32px",
      }}
    />
  );
};
