"use client";

// React Imports
import React, {
  createContext,
  useContext,
  useCallback,
  useState,
  useRef,
  useEffect,
} from "react";
import { usePageNavigation } from "./PageNavigationContext";
import { usePageTree } from "../../../../contexts/PageTreeContext";

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
  /** Horizontal position relative to parent container */
  leftPosition: number | null;
  /** Whether the tooltip should be visible */
  visible: boolean;
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
  /** Reference to the target element for tooltip positioning */
  tooltipTargetRef: React.RefObject<HTMLDivElement | null>;
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
  // Extract the page tree context
  const { getPageByTitle } = usePageTree();

  // Extract the page navigation context
  const { focusedPageTitle } = usePageNavigation();

  // Initial tooltip state
  const [tooltipProps, setTooltipProps] = useState<TooltipProps>({
    content: {
      title: "",
      isUrl: false,
      isFolder: false,
    },
    appearance: {
      leftPosition: null,
      topPosition: null,
      visible: false,
      animationDirection: null,
    },
  });

  // Ref for managing tooltip target element
  const tooltipTargetRef = useRef<HTMLDivElement | null>(null);

  // Ref for managing hide animation timeout
  const hideTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Ref for managing content container element
  const contentContainerRef = useRef<HTMLDivElement>(null);

  // Ref for managing current content container element
  const currentContentContainerRef = useRef<HTMLDivElement>(null);

  const showTooltip = useCallback(
    ({ newContent }: { newContent: TooltipContent }) => {
      // Cancel any pending hide animations
      if (hideTimeoutRef.current) {
        clearTimeout(hideTimeoutRef.current);
      }

      // Calculate and update tooltip position
      if (tooltipTargetRef.current) {
        // Get the bounding rectangle of the tooltip target
        const targetRect = tooltipTargetRef.current.getBoundingClientRect();

        setTooltipProps((prev) => ({
          previousContent: prev.content,
          content: newContent,
          appearance: {
            topPosition: targetRect.top,
            leftPosition: targetRect.right,
            visible: true,
            animationDirection:
              !targetRect.top ||
              !prev.appearance.topPosition ||
              targetRect.top === prev.appearance.topPosition
                ? null
                : targetRect.top > prev.appearance.topPosition
                ? "down"
                : "up",
          },
        }));
      }
    },
    []
  );

  const hideTooltip = useCallback(() => {
    // Cancel any existing hide animations
    if (hideTimeoutRef.current) {
      clearTimeout(hideTimeoutRef.current);
    }

    // Start new hide animation
    hideTimeoutRef.current = setTimeout(() => {
      setTooltipProps((prev) => ({
        content: prev.content,
        previousContent: prev.previousContent,
        appearance: {
          ...prev.appearance,
          visible: false,
        },
      }));
    }, 150);
  }, []);

  // Transform the tooltip when the focused page changes
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
      newContent: {
        title: focusedPage.metadata.title,
        description: focusedPage.metadata.description,
        isUrl: focusedPage.url !== undefined,
        isFolder: focusedPage.children !== undefined,
      },
    });
  }, [focusedPageTitle, showTooltip, hideTooltip, getPageByTitle]);

  // Update parent container height when child content changes
  useEffect(() => {
    if (!currentContentContainerRef.current) return;

    // Observe changes in height of the current content
    const resizeObserver = new ResizeObserver((entries) => {
      if (!contentContainerRef.current) return;
      for (const entry of entries) {
        // Update the height of the parent container to match the current content
        contentContainerRef.current.style.height = `${entry.contentRect.height}px`;
      }
    });

    resizeObserver.observe(currentContentContainerRef.current);
    return () => resizeObserver.disconnect();
  }, [tooltipProps.content]);

  return (
    <TooltipContext.Provider
      value={{
        tooltipProps,
        tooltipTargetRef,
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
