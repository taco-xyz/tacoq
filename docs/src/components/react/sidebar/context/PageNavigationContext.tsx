"use client";

// React Imports
import {
  createContext,
  useContext,
  useState,
  useCallback,
  useEffect,
  useRef,
  RefObject,
  useLayoutEffect,
  PropsWithChildren,
} from "react";

// Next Imports
import { useRouter } from "next/navigation";

// Context Imports
import { usePageTree } from "../../../../contexts/PageTreeContext";

/**
 * Context for managing page tree element navigation and focus state
 */
interface PageNavigationContextType {
  /** Focus a specific page tree element by its title when it's hovered */
  startHoverFocus: (elementTitle: string) => void;
  /** Start keyboard focus mode on the sidebar, focusing current page or first available */
  startKeyboardFocus: () => void;
  /** End keyboard focus mode on the sidebar */
  endKeyboardFocus: () => void;
  /** End hover focus on the sidebar */
  endHoverFocus: () => void;
  /** Title of currently focused page tree element, or null if none focused */
  focusedElementTitle: string | null;
  /** Ref to the page tree element container element, to unfocus the page tree element navigation when clicking outside of it's container */
  elementContainerRef: RefObject<HTMLDivElement | null>;
  /** Ref to the sidebar container element, to avoid focusing page tree elements that are hidden by this element's scroll */
  sidebarContainerRef: RefObject<HTMLDivElement | null>;
  /** Ref to the current focused page tree element*/
  currentFocusedElementRef: RefObject<HTMLDivElement | null>;
}

const PageNavigationContext = createContext<PageNavigationContextType | null>(
  null,
);

export function PageNavigationProvider({ children }: PropsWithChildren) {
  const router = useRouter();

  const elementContainerRef = useRef<HTMLDivElement>(null);

  const sidebarContainerRef = useRef<HTMLDivElement>(null);

  const currentFocusedElementRef = useRef<HTMLDivElement>(null);

  // Extract page tree context
  const {
    visibleElementsTitles,
    currentElementTitle,
    getElementByTitle,
    isFolder,
    isFolderExpanded,
    expandFolder,
    collapseFolder,
  } = usePageTree();

  /** Title of currently focused page tree element, or null if none focused */
  const [focusedElementTitle, setFocusedElementTitle] = useState<string | null>(
    null,
  );

  /** Ref for managing hover focus timeout */
  const hoverFocusTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * Focus a specific page tree element by its title
   */
  const startHoverFocus = useCallback((elementTitle: string) => {
    if (hoverFocusTimeoutRef.current) {
      clearTimeout(hoverFocusTimeoutRef.current);
    }

    setFocusedElementTitle(elementTitle);
  }, []);

  /**
   * Handle hover focus on the sidebar
   */
  const endHoverFocus = useCallback(() => {
    if (!focusedElementTitle) return;

    if (hoverFocusTimeoutRef.current) {
      clearTimeout(hoverFocusTimeoutRef.current);
    }

    hoverFocusTimeoutRef.current = setTimeout(() => {
      setFocusedElementTitle(null);
    }, 150);
  }, [focusedElementTitle]);

  /**
   * Start keyboard focus mode on the sidebar
   * If no element is focused, focuses either the current page tree element or first available element
   */
  const startKeyboardFocus = useCallback(() => {
    if (focusedElementTitle || visibleElementsTitles.length === 0) return;

    const currentElementIndex = currentElementTitle
      ? visibleElementsTitles.indexOf(currentElementTitle)
      : -1;

    // Set the focused page to the current page or the first available page
    setFocusedElementTitle(
      currentElementIndex >= 0 ? currentElementTitle : visibleElementsTitles[0],
    );
  }, [visibleElementsTitles, currentElementTitle, focusedElementTitle]);

  /**
   * End keyboard focus mode on the sidebar by clearing focused page
   */
  const endKeyboardFocus = useCallback(() => {
    if (!focusedElementTitle) return;
    setFocusedElementTitle(null);
  }, [focusedElementTitle]);

  /**
   * Exit focus mode when clicking outside page tree element container
   */
  const handleClickOutside = useCallback(
    (e: MouseEvent) => {
      if (
        !elementContainerRef.current ||
        elementContainerRef.current.contains(e.target as Node)
      )
        return;

      endKeyboardFocus();
    },
    [endKeyboardFocus],
  );

  /**
   * Handle keyboard navigation in the sidebar
   * - Ctrl+0 (or Cmd+0 on Mac): Start focus mode
   * - Arrow Up/Down: Navigate between visible page tree elements
   * - Space/Arrow Right/Left: Expand/collapse page tree elements with children
   * - Enter: Navigate to page tree element URL
   * - Escape: Exit focus mode
   */
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "0") {
        e.preventDefault();
        startKeyboardFocus();
        return;
      }

      if (!focusedElementTitle) return;

      const currentIndex = visibleElementsTitles.indexOf(focusedElementTitle);
      const currentElement = getElementByTitle(focusedElementTitle);

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          // Check if the current element is not the last page tree element
          if (currentIndex < visibleElementsTitles.length - 1) {
            // Set the focused element to the next page tree element
            setFocusedElementTitle(visibleElementsTitles[currentIndex + 1]);
            return;
          } else {
            // Set the focused element to the first page tree element
            setFocusedElementTitle(visibleElementsTitles[0]);
            return;
          }

        case "ArrowUp":
          e.preventDefault();
          // Check if the current page is not the first page
          if (currentIndex > 0) {
            // Set the focused element to the previous page tree element
            setFocusedElementTitle(visibleElementsTitles[currentIndex - 1]);
            return;
          } else {
            // Set the focused element to the last page tree element
            setFocusedElementTitle(
              visibleElementsTitles[visibleElementsTitles.length - 1],
            );
            return;
          }

        case " ":
          e.preventDefault();
          // Check if the current element is a folder
          if (!currentElement || !isFolder(currentElement)) return;

          // Toggle the expansion state of the folder
          if (isFolderExpanded(focusedElementTitle)) {
            collapseFolder(focusedElementTitle);
            return;
          } else {
            expandFolder(focusedElementTitle);
            return;
          }

        case "ArrowRight":
          e.preventDefault();
          // Check if the current element is a folder and is expanded
          if (
            !currentElement ||
            !isFolder(currentElement) ||
            isFolderExpanded(focusedElementTitle)
          )
            return;

          expandFolder(focusedElementTitle);
          return;

        case "ArrowLeft":
          e.preventDefault();
          // Check if the current element is a folder and is expanded
          if (
            !currentElement ||
            !isFolder(currentElement) ||
            !isFolderExpanded(focusedElementTitle)
          )
            return;

          collapseFolder(focusedElementTitle);
          return;

        case "Enter":
          e.preventDefault();
          // Check if the current element is a not a folder
          if (!currentElement || isFolder(currentElement)) return;

          // Navigate to the current element's URL
          router.push(currentElement.url);
          return;

        case "Escape":
          e.preventDefault();
          endKeyboardFocus();
          return;
      }
    },
    [
      focusedElementTitle,
      visibleElementsTitles,
      expandFolder,
      collapseFolder,
      isFolderExpanded,
      getElementByTitle,
      router,
      startKeyboardFocus,
      endKeyboardFocus,
      isFolder,
    ],
  );

  // Scroll to current focused element if it's hidden by the sidebar scroll
  useLayoutEffect(() => {
    if (
      !focusedElementTitle ||
      !currentFocusedElementRef.current ||
      !sidebarContainerRef.current
    )
      return;

    // Get the current focused page element and the sidebar container element
    const pageElement = currentFocusedElementRef.current;
    const sidebarElement = sidebarContainerRef.current;

    const pageRect = pageElement.getBoundingClientRect();
    const sidebarRect = sidebarElement.getBoundingClientRect();

    // Check if page is below the sidebar container
    if (pageRect.bottom > sidebarRect.bottom) {
      sidebarElement.scrollBy({
        top: pageRect.bottom - sidebarRect.bottom + 18, // Overscroll slightly
      });
    }

    // Check if page is above the sidebar container
    if (pageRect.top < sidebarRect.top) {
      sidebarElement.scrollBy({
        top: pageRect.top - sidebarRect.top - 18, // Overscroll slightly
      });
    }
  }, [focusedElementTitle]);

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [handleKeyDown, handleClickOutside]);

  return (
    <PageNavigationContext.Provider
      value={{
        startHoverFocus,
        startKeyboardFocus,
        endKeyboardFocus,
        endHoverFocus,
        focusedElementTitle,
        elementContainerRef,
        sidebarContainerRef,
        currentFocusedElementRef,
      }}
    >
      {children}
    </PageNavigationContext.Provider>
  );
}

export function usePageNavigation() {
  const context = useContext(PageNavigationContext);
  if (!context) {
    throw new Error(
      "usePageNavigation must be used within PageNavigationProvider",
    );
  }
  return context;
}
