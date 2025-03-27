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
  // Extract the page tree context
  const { currentPageTitle, isPageExpanded } = usePageTree();

  // Checks if the current item or any of its descendants is selected
  const isSelfOrDescendantSelected = useCallback(
    (item: Page): boolean => {
      // Check if the item is selected
      if (item.metadata.title === currentPageTitle) return true;
      // Recursively check if any of the item's descendants are selected
      if (item.children) {
        return (
          item.children &&
          item.children.some((child) => isSelfOrDescendantSelected(child))
        );
      }
      return false;
    },
    [currentPageTitle],
  );

  // An item's child should be focused if it's selected or if any of it's descendants are selected
  // Returns -1 if no children should be focused
  // If a child should be focused, returns the index of the child
  const focusedDirectChildIndex = useMemo(() => {
    if (!children) return -1;

    return children.findIndex((child) => isSelfOrDescendantSelected(child));
  }, [children, isSelfOrDescendantSelected]);

  // Updates the highlight position
  const updateHighlightPosition = useCallback(() => {
    // The item has a child that should be focused
    if (focusedDirectChildIndex !== -1 && parentElementRef.current) {
      // Fetches all the child elements of the current item with the "data-child-of" attribute
      const childElements = parentElementRef.current.querySelectorAll(
        `[data-child-of="${title}"]`,
      );
      // Fetches the child element that should be focused through index
      const focusedElement = childElements[focusedDirectChildIndex];
      if (focusedElement) {
        // Fetches the parent element's position
        const parentRect = parentElementRef.current.getBoundingClientRect();
        // Fetches the child element's position
        const childRect = focusedElement.getBoundingClientRect();
        // Returns the child element's top position relative to the parent element
        return childRect.top - parentRect.top;
      }
    }
    return null;
  }, [focusedDirectChildIndex, title, parentElementRef]);

  const [highlightPosition, setHighlightPosition] = useState<number | null>(
    updateHighlightPosition(),
  );

  // Updates the highlight position when a new element is focused
  useEffect(() => {
    setHighlightPosition(updateHighlightPosition());
  }, [updateHighlightPosition]);

  return (
    <div
      className={clsx(
        "absolute left-[14px] h-7 w-[0.5px] rounded-full bg-zinc-800 shadow-xs shadow-zinc-900/25 transition-all duration-150 ease-in-out dark:bg-white dark:shadow-white/15",
        focusedDirectChildIndex !== -1 && highlightPosition !== null
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
