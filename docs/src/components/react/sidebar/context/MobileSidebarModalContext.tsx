"use client";

// React Imports
import {
  FC,
  createContext,
  useContext,
  useCallback,
  useEffect,
  useState,
  RefObject,
  useRef,
  PropsWithChildren,
} from "react";

/**
 * Interface for the MobileSidebarModalContextType
 * @property {boolean} isSidebarOpen - Indicates if the sidebar is open
 * @property {() => void} openSidebar - Function to open the sidebar
 * @property {() => void} closeSidebar - Function to close the sidebar
 * @property {RefObject<HTMLDivElement | null>} dialogRef - Reference to the sidebar dialog element
 */
interface MobileSidebarModalContextType {
  isSidebarOpen: boolean;
  openSidebar: () => void;
  closeSidebar: () => void;
  dialogRef: RefObject<HTMLDivElement | null>;
}

/**
 * Creates a context for managing the mobile sidebar state
 */
const MobileSidebarModalContext = createContext<
  MobileSidebarModalContextType | undefined
>(undefined);

/**
 * Provider component for the MobileSidebarContext
 * @param {PropsWithChildren} children - The children components to render within the context
 */
export const MobileSidebarModalProvider: FC<PropsWithChildren> = ({
  children,
}) => {
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const dialogRef = useRef<HTMLDivElement>(null);

  /**
   * Function to open the sidebar
   */
  const openSidebar = useCallback(() => setIsSidebarOpen(true), []);

  /**
   * Function to close the sidebar
   */
  const closeSidebar = useCallback(() => setIsSidebarOpen(false), []);

  /**
   * Close the sidebar when clicking outside of it
   */
  const handleClickOutside = useCallback(
    (e: MouseEvent) => {
      if (!dialogRef.current || dialogRef.current.contains(e.target as Node))
        return;

      closeSidebar();
    },
    [closeSidebar, dialogRef],
  );

  // Listen for click events
  useEffect(() => {
    if (!isSidebarOpen) return;

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isSidebarOpen, handleClickOutside]);

  // Handle scroll lock and padding
  useEffect(() => {
    if (!isSidebarOpen) return;

    // Get the current width of the scrollbar
    const scrollbarWidth =
      window.innerWidth - document.documentElement.clientWidth;

    // Get the current scroll position
    const scrollY = window.scrollY;

    // Compensate for the scrollbar width
    if (scrollbarWidth > 0) {
      document.body.style.paddingRight = `${scrollbarWidth}px`;
    }

    // Lock scroll while preserving position (prevents sticky elements from behaving weirdly)
    document.body.style.position = "fixed";
    document.body.style.top = `-${scrollY}px`;

    return () => {
      // Restore scroll position
      document.body.style.position = "";
      document.body.style.top = "";
      document.body.style.paddingRight = "";
      window.scrollTo(0, scrollY);
    };
  }, [isSidebarOpen]);

  return (
    <MobileSidebarModalContext.Provider
      value={{ isSidebarOpen, openSidebar, closeSidebar, dialogRef }}
    >
      {children}
    </MobileSidebarModalContext.Provider>
  );
};

/**
 * Hook to use the MobileSidebarContext
 * Throws an error if used outside of the MobileSidebarProvider
 */
export function useMobileSidebarModal() {
  const context = useContext(MobileSidebarModalContext);
  if (!context)
    throw new Error(
      "useMobileSidebarModal must be used within MobileSidebarModalProvider",
    );
  return context;
}
