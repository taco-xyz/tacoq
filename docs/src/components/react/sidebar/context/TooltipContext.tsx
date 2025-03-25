"use client";

// React Imports
import React, {
  createContext,
  useContext,
  useCallback,
  useState,
  useRef,
  useEffect,
  useLayoutEffect,
  useMemo,
} from "react";

// Context Imports
import { usePageNavigation } from "./PageNavigationContext";
import { usePageTree } from "../../../../contexts/PageTreeContext";

// Next Imports
import { usePathname } from "next/navigation";

/** Content to display in the tooltip */
export interface TooltipContent {
  /** Title text shown in tooltip */
  title: string;
  /** Optional description text shown in tooltip */
  description?: string;
  /** Whether the page the tooltip is currently displaying has an url */
  isUrl: boolean;
  /** Whether the tooltip is for a folder */
  isFolder: boolean;
}

export interface TooltipAppearance {
  /** Vertical position relative to parent container */
  topPosition: number | null;
  /** Wether the tooltip should be positioned at the top or bottom of the tooltip*/
  arrowPosition: "top" | "bottom" | null;
  /** Horizontal position relative to parent container */
  leftPosition: number | null;
  /** Opacity of the tooltip */
  opacity: 100 | 50 | 0;
  /** Direction of the tooltip animation */
  animationDirection: "up" | "down" | null;
}

/**
 * Props for controlling tooltip appearance
 */
export interface TooltipProps {
  /** Content to display in the tooltip */
  content: TooltipContent;
  /** Previous content to display in the tooltip */
  previousContent?: TooltipContent;
  /** Appearance properties for the tooltip */
  appearance: TooltipAppearance;
}

/**
 * Context for managing tooltip state and behavior

 */
export interface TooltipContextType {
  /** Current tooltip appearance properties */
  tooltipProps: TooltipProps;
  /** Reference to the content container element for dynamic height transition*/
  contentContainerRef: React.RefObject<HTMLDivElement | null>;
  /** Reference to the current content container element for dynamic height transition*/
  currentContentContainerRef: React.RefObject<HTMLDivElement | null>;
}

/** Context for sharing tooltip state and methods */
const TooltipContext = createContext<TooltipContextType | null>(null);

/**
 * Provider component for tooltip functionality
 * Manages tooltip state and positioning logic
 *
 * @example
 * ```tsx
 * <TooltipProvider>
 *   <App />
 * </TooltipProvider>
 * ```
 */
export function TooltipProvider({ children }: { children: React.ReactNode }) {
  // Extract the pathname
  const pathname = usePathname();

  // Extract the page tree context
  const { getPageByTitle, visiblePagesTitles } = usePageTree();

  // Extract the page navigation context
  const {
    focusedPageTitle,
    sidebarContainerRef,
    currentFocusedPageRef,
    endKeyboardFocus,
  } = usePageNavigation();

  // State to manage the tooltip content, including both current and previous content
  const [contentState, setContentState] = useState<{
    current: TooltipContent;
    previous?: TooltipContent;
  }>({
    current: {
      title: "",
      isUrl: false,
      isFolder: false,
    },
  });

  // State to manage the tooltip appearance
  const [appearanceState, setAppearanceState] = useState<TooltipAppearance>({
    leftPosition: null,
    topPosition: null,
    arrowPosition: null,
    opacity: 0,
    animationDirection: null,
  });

  // Combine the content and appearance state into a single object
  const tooltipProps = useMemo(
    () => ({
      content: contentState.current,
      previousContent: contentState.previous,
      appearance: appearanceState,
    }),
    [contentState, appearanceState]
  );

  // Ref for managing hide animation timeout
  const hideTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Ref for managing content container element
  const contentContainerRef = useRef<HTMLDivElement>(null);

  // Ref for managing current content container element
  const currentContentContainerRef = useRef<HTMLDivElement>(null);

  const showTooltip = useCallback((newContent: TooltipContent) => {
    // Cancel any existing hide animations
    if (hideTimeoutRef.current) {
      clearTimeout(hideTimeoutRef.current);
    }
    // Set the new content and make the tooltip visible
    setContentState((prev) => ({
      previous: prev.current,
      current: newContent,
    }));

    setAppearanceState((prev) => ({
      ...prev,
      opacity: 100,
    }));
  }, []);

  const hideTooltip = useCallback(() => {
    // Cancel any existing hide animations
    if (hideTimeoutRef.current) {
      clearTimeout(hideTimeoutRef.current);
    }
    // Hide tooltip after a delay
    hideTimeoutRef.current = setTimeout(() => {
      // Hide tooltip
      setAppearanceState((prev) => ({
        ...prev,
        opacity: 0,
      }));
    }, 150);
  }, []);

  const hideTooltipInstantly = useCallback(() => {
    // Cancel any existing hide animations
    if (hideTimeoutRef.current) {
      clearTimeout(hideTimeoutRef.current);
    }
    // Hide tooltip immediately
    setAppearanceState((prev) => ({
      ...prev,
      opacity: 0,
    }));
    // End keyboard focus mode
    endKeyboardFocus();
  }, [endKeyboardFocus]);

  // Hide the tooltip instantly when scrolling the whole page or wheelling the sidebar container to avoid weird overlapping behavior
  useEffect(() => {
    const sidebarRef = sidebarContainerRef.current;
    sidebarRef?.addEventListener("wheel", hideTooltipInstantly);

    window.addEventListener("scroll", hideTooltipInstantly);
    return () => {
      sidebarRef?.removeEventListener("wheel", hideTooltipInstantly);
      window.removeEventListener("scroll", hideTooltipInstantly);
    };
  }, [hideTooltipInstantly, sidebarContainerRef]);

  // Show/Hide the tooltip when the focused page changes
  useEffect(() => {
    if (!focusedPageTitle) {
      hideTooltip();
      return;
    }

    const focusedPage = getPageByTitle(focusedPageTitle);
    if (!focusedPage) {
      hideTooltip();
      return;
    }

    showTooltip({
      title: focusedPage.metadata.title,
      description: focusedPage.metadata.description,
      isUrl: focusedPage.url !== undefined,
      isFolder: focusedPage.children !== undefined,
    });
  }, [focusedPageTitle, showTooltip, hideTooltip, getPageByTitle]);

  // Set the tooltip opacity to 50 when the pathname changes in order to better see the new page
  // If the tooltip was hidden, keep it hidden
  useEffect(() => {
    setAppearanceState((prev) => ({
      ...prev,
      opacity: prev.opacity === 0 ? 0 : 50,
    }));
  }, [pathname]);

  // Update tooltip appearance when it's content changes or when it's set to visible
  useLayoutEffect(() => {
    if (!currentContentContainerRef.current) return;

    // Create a resize observer to monitor the height of the tooltip content
    const resizeObserver = new ResizeObserver((entries) => {
      // Return if any of the dependencies are not available or if the tooltip is not visible
      if (
        !currentFocusedPageRef.current ||
        !sidebarContainerRef.current ||
        !contentContainerRef.current ||
        appearanceState.opacity === 0
      ) {
        return;
      }

      for (const entry of entries) {
        // Measure the new height of the tooltip content and update it to ensure a smooth height transition
        const height = entry.contentRect.height;
        contentContainerRef.current.style.height = `${height}px`;

        // Get the position of the target page element and the sidebar container element
        const targetRect =
          currentFocusedPageRef.current.getBoundingClientRect();
        const sidebarRect = sidebarContainerRef.current.getBoundingClientRect();

        // Calculate the whole height of the tooltip including the fixed height content (Hot-keys)
        const tooltipFixedContentHeight = 60;
        const tooltipHeight = height + tooltipFixedContentHeight;

        // Calculate if sidebar overflow happens if the tooltip is positioned relative to the top of the target element
        const wouldOverflow =
          targetRect.top + tooltipHeight >= sidebarRect.bottom;

        // Calculate the new top position of the tooltip
        const newTopPosition = wouldOverflow
          ? targetRect.bottom - tooltipHeight + 1
          : targetRect.top + 1;

        const getAnimationDirection = (
          currentTitle: string,
          previousTitle?: string
        ) => {
          //If there's no previous top position we're focusing for the first time on the tooltip
          if (!previousTitle || currentTitle === previousTitle) return null;

          // Fetch the index of the current and previous focused page
          const currentIndex = visiblePagesTitles.indexOf(currentTitle);
          const previousIndex = visiblePagesTitles.indexOf(previousTitle);

          // If the current index is greater than the previous index, the tooltip should animate down
          if (currentIndex > previousIndex) return "down";
          //If the new top position is above the previous top position, the tooltip should animate up
          return "up";
        };

        // Update positioning in state
        setAppearanceState((prev) => ({
          ...prev,
          // If the tooltip would overflow, position it at the bottom of the target element, otherwise position it at the top
          topPosition: newTopPosition,
          // If the tooltip would overflow, position the arrow at the bottom of the tooltip, otherwise position it at the top
          arrowPosition: wouldOverflow ? "bottom" : "top",
          // Position the tooltip to the right of the target element with a small offset
          leftPosition: targetRect.right + 16,
          // Set the direction of the tooltip animation
          animationDirection: getAnimationDirection(
            contentState.current.title,
            contentState.previous?.title
          ),
        }));
      }
    });

    resizeObserver.observe(currentContentContainerRef.current);
    return () => resizeObserver.disconnect();
  }, [
    contentState,
    appearanceState.opacity,
    currentFocusedPageRef,
    sidebarContainerRef,
    visiblePagesTitles,
  ]);

  return (
    <TooltipContext.Provider
      value={{
        tooltipProps,
        contentContainerRef,
        currentContentContainerRef,
      }}
    >
      {children}
    </TooltipContext.Provider>
  );
}

/**
 * Hook for accessing tooltip context
 * @throws Error if used outside of TooltipProvider
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { tooltipProps, showTooltip, hideTooltip } = useTooltip();
 *   // ...
 * }
 * ```
 */
export function useTooltip() {
  const context = useContext(TooltipContext);
  if (!context) {
    throw new Error("useTooltip must be used within TooltipProvider");
  }
  return context;
}
