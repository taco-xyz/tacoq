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
 * Context for managing page navigation and focus state
 */
interface PageNavigationContextType {
  /** Focus a specific page by its title when it's hovered */
  startHoverFocus: (pageTitle: string) => void;
  /** Start keyboard focus mode on the sidebar, focusing current page or first available */
  startKeyboardFocus: () => void;
  /** End keyboard focus mode on the sidebar */
  endKeyboardFocus: () => void;
  /** End hover focus on the sidebar */
  endHoverFocus: () => void;
  /** Title of currently focused page, or null if none focused */
  focusedPageTitle: string | null;
  /** Ref to the page container element, to unfocus the page navigation when clicking outside of it's container */
  pageContainerRef: RefObject<HTMLDivElement | null>;
  /** Ref to the sidebar container element, to avoid focusing pages that are hidden by this element's scroll */
  sidebarContainerRef: RefObject<HTMLDivElement | null>;
  /** Ref to the current focused page element*/
  currentFocusedPageRef: RefObject<HTMLDivElement | null>;
}

const PageNavigationContext = createContext<PageNavigationContextType | null>(
  null,
);

export function PageNavigationProvider({ children }: PropsWithChildren) {
  const router = useRouter();

  const pageContainerRef = useRef<HTMLDivElement>(null);

  const sidebarContainerRef = useRef<HTMLDivElement>(null);

  const currentFocusedPageRef = useRef<HTMLDivElement>(null);

  // Extract page tree context
  const {
    visiblePagesTitles,
    currentPageTitle,
    getPageByTitle,
    expandPage,
    collapsePage,
    isPageExpanded,
  } = usePageTree();

  /** Title of currently focused page, or null if none focused */
  const [focusedPageTitle, setFocusedPageTitle] = useState<string | null>(null);

  /** Ref for managing hover focus timeout */
  const hoverFocusTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * Focus a specific page by its title
   */
  const startHoverFocus = useCallback((pageTitle: string) => {
    if (hoverFocusTimeoutRef.current) {
      clearTimeout(hoverFocusTimeoutRef.current);
    }

    setFocusedPageTitle(pageTitle);
  }, []);

  /**
   * Handle hover focus on the sidebar
   */
  const endHoverFocus = useCallback(() => {
    if (focusedPageTitle) {
      if (hoverFocusTimeoutRef.current) {
        clearTimeout(hoverFocusTimeoutRef.current);
      }

      hoverFocusTimeoutRef.current = setTimeout(() => {
        setFocusedPageTitle(null);
      }, 150);
    }
  }, [focusedPageTitle]);

  /**
   * Start keyboard focus mode on the sidebar
   * If no page is focused, focuses either the current page or first available page
   */
  const startKeyboardFocus = useCallback(() => {
    if (!focusedPageTitle && visiblePagesTitles.length > 0) {
      // Get the current page index
      const currentPageIndex = currentPageTitle
        ? visiblePagesTitles.indexOf(currentPageTitle)
        : -1;

      // Set the focused page to the current page or the first available page
      setFocusedPageTitle(
        currentPageIndex >= 0 ? currentPageTitle : visiblePagesTitles[0],
      );
    }
  }, [visiblePagesTitles, currentPageTitle, focusedPageTitle]);

  /**
   * End keyboard focus mode on the sidebar by clearing focused page
   */
  const endKeyboardFocus = useCallback(() => {
    if (focusedPageTitle) {
      setFocusedPageTitle(null);
    }
  }, [focusedPageTitle]);

  /**
   * Exit focus mode when clicking outside page container

   */
  const handleClickOutside = useCallback(
    (e: MouseEvent) => {
      if (
        pageContainerRef.current &&
        !pageContainerRef.current.contains(e.target as Node)
      ) {
        endKeyboardFocus();
      }
    },
    [endKeyboardFocus],
  );

  /**
   * Handle keyboard navigation in the sidebar
   * - Ctrl+0 (or Cmd+0 on Mac): Start focus mode
   * - Arrow Up/Down: Navigate between visible pages
   * - Space/Arrow Right/Left: Expand/collapse pages with children
   * - Enter: Navigate to page URL
   * - Escape: Exit focus mode
   */
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "0") {
        e.preventDefault();
        startKeyboardFocus();
        return;
      }

      if (!focusedPageTitle) return;

      const currentIndex = visiblePagesTitles.indexOf(focusedPageTitle);
      const currentPage = getPageByTitle(focusedPageTitle);

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          // Check if the current page is not the last page
          if (currentIndex < visiblePagesTitles.length - 1) {
            // Set the focused page to the next page
            setFocusedPageTitle(visiblePagesTitles[currentIndex + 1]);
          } else {
            // Set the focused page to the first page
            setFocusedPageTitle(visiblePagesTitles[0]);
          }
          break;

        case "ArrowUp":
          e.preventDefault();
          // Check if the current page is not the first page
          if (currentIndex > 0) {
            // Set the focused page to the previous page
            setFocusedPageTitle(visiblePagesTitles[currentIndex - 1]);
          } else {
            // Set the focused page to the last page
            setFocusedPageTitle(
              visiblePagesTitles[visiblePagesTitles.length - 1],
            );
          }
          break;

        case " ":
          e.preventDefault();
          if (currentPage?.children) {
            if (isPageExpanded(focusedPageTitle)) {
              collapsePage(focusedPageTitle);
            } else {
              expandPage(focusedPageTitle);
            }
          }
          break;

        case "ArrowRight":
          e.preventDefault();
          if (currentPage?.children && !isPageExpanded(focusedPageTitle)) {
            expandPage(focusedPageTitle);
          }
          break;

        case "ArrowLeft":
          e.preventDefault();
          if (currentPage?.children && isPageExpanded(focusedPageTitle)) {
            collapsePage(focusedPageTitle);
          }
          break;

        case "Enter":
          e.preventDefault();
          if (currentPage?.url) {
            router.push(currentPage.url);
          }
          break;

        case "Escape":
          e.preventDefault();
          endKeyboardFocus();
          break;
      }
    },
    [
      focusedPageTitle,
      visiblePagesTitles,
      expandPage,
      collapsePage,
      isPageExpanded,
      getPageByTitle,
      router,
      startKeyboardFocus,
      endKeyboardFocus,
    ],
  );

  // Scroll to current focused page if it's hidden by the sidebar scroll
  useLayoutEffect(() => {
    if (
      !focusedPageTitle ||
      !currentFocusedPageRef.current ||
      !sidebarContainerRef.current
    )
      return;

    // Get the current focused page element and the sidebar container element
    const pageElement = currentFocusedPageRef.current;
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
  }, [focusedPageTitle]);

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
        focusedPageTitle,
        pageContainerRef,
        sidebarContainerRef,
        currentFocusedPageRef,
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
