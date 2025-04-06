"use client";

// React Imports
import {
  FC,
  createContext,
  useContext,
  useCallback,
  useEffect,
  useState,
  useRef,
  RefObject,
  PropsWithChildren,
} from "react";

/**
 * Interface for the SearchModalContextType
 * @property {boolean} isSearchOpen - Indicates if the search is currently open
 * @property {() => void} openSearch - Function to open the search
 * @property {() => void} closeSearch - Function to close the search
 * @property {RefObject<HTMLDivElement | null>} dialogRef - Ref for the dialog element (handles click events outside of the search modal)
 * @property {RefObject<HTMLInputElement | null>} inputRef - Ref for the input element (focuses the input when the search modal is opened)
 */
interface SearchModalContextType {
  isSearchOpen: boolean;
  openSearch: () => void;
  closeSearch: () => void;
  dialogRef: RefObject<HTMLDivElement | null>;
  inputRef: RefObject<HTMLInputElement | null>;
}

/**
 * Creates a context for managing the search state
 */
const SearchModalContext = createContext<SearchModalContextType | undefined>(
  undefined,
);

/**
 * Provider component for the SearchModalContext
 * @param {PropsWithChildren} children - The children components to be wrapped by the provider
 */
export const SearchModalProvider: FC<PropsWithChildren> = ({ children }) => {
  const [isSearchOpen, setIsSearchOpen] = useState(false);

  // Ref for the dialog element, used to listen for click events outside of it
  const dialogRef = useRef<HTMLDivElement>(null);
  // Ref for the input element, used to focus it automaticallywhen the search modal is opened
  const inputRef = useRef<HTMLInputElement>(null);

  /**
   * Function to open the search
   * Clears the input value and opens the search
   */
  const openSearch = useCallback(() => {
    if (!inputRef.current) return;

    inputRef.current.value = "";
    setIsSearchOpen(true);
  }, []);

  /**
   * Function to close the search
   */
  const closeSearch = useCallback(() => setIsSearchOpen(false), []);

  // Handle keyboard shortcuts
  // Ctrl + K (or Cmd + K on Mac) to open the search
  // Esc to close the search
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      switch (e.key) {
        case "k":
          if (isSearchOpen || (!e.ctrlKey && !e.metaKey)) return;

          e.preventDefault();
          openSearch();
          return;
        case "Escape":
          if (!isSearchOpen) return;

          e.preventDefault();
          closeSearch();
          return;
      }
    },
    [isSearchOpen, openSearch, closeSearch],
  );

  // Close the search modal when clicking outside of the search panel
  const handleClickOutside = useCallback(
    (e: MouseEvent) => {
      if (!dialogRef.current || dialogRef.current.contains(e.target as Node))
        return;

      closeSearch();
    },
    [closeSearch],
  );

  // Event Listeners for keyboard shortcuts
  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  // Event listener for clicks
  useEffect(() => {
    if (!isSearchOpen) return;

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isSearchOpen, handleClickOutside]);

  // Handle scroll lock and padding
  useEffect(() => {
    if (!isSearchOpen) return;

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

    // Focus the input element
    inputRef.current?.focus();

    return () => {
      // Restore scroll position
      document.body.style.position = "";
      document.body.style.top = "";
      document.body.style.paddingRight = "";
      window.scrollTo(0, scrollY);
    };
  }, [isSearchOpen]);

  return (
    <SearchModalContext.Provider
      value={{ isSearchOpen, openSearch, closeSearch, dialogRef, inputRef }}
    >
      {children}
    </SearchModalContext.Provider>
  );
};

/**
 * Hook to use the SearchModalContext
 * Throws an error if used outside of SearchModalProvider
 * @returns {SearchModalContextType} The context value
 */
export function useSearchModal() {
  const context = useContext(SearchModalContext);
  if (!context)
    throw new Error("useSearchModal must be used within SearchModalProvider");
  return context;
}
