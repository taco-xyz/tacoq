"use client";

// React Imports
import { useEffect, useCallback, useState, useRef } from "react";
import { createPortal } from "react-dom";

// Next Imports
import dynamic from "next/dynamic";

// Heroicons Imports
import { MagnifyingGlassIcon } from "@heroicons/react/24/outline";

// Utils Imports
import clsx from "clsx";

// Components Imports
const SearchPortal = dynamic<{ children: React.ReactNode }>(
  () =>
    Promise.resolve(({ children }: { children: React.ReactNode }) =>
      createPortal(children, document.body)
    ),
  {
    ssr: false,
  }
);

/**
 * Search component that provides a search interface for documentation
 * Features:
 * - Keyboard shortcut (Ctrl+K) to open search
 * - Escape to close
 * - Click outside to dismiss
 * - Search input with autofocus
 */
export default function Search() {
  const [isSearchOpen, setIsSearchOpen] = useState(false);

  // Ref for the dialog element, used to listen for click events outside of it
  const dialogRef = useRef<HTMLDivElement>(null);
  // Ref for the input element, used to focus it automaticallywhen the search modal is opened
  const inputRef = useRef<HTMLInputElement>(null);

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

  // Close the search modal when clicking outside of the search panel
  const handleClickOutside = useCallback((e: MouseEvent) => {
    if (dialogRef.current && !dialogRef.current.contains(e.target as Node)) {
      setIsSearchOpen(false);
    }
  }, []);

  // Open the search modal when the user presses Ctrl+K
  // Close the search modal when the user presses Esc
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k" && !isSearchOpen) {
        e.preventDefault(); // Default browser behaviour is to open the search bar
        setIsSearchOpen(true);
      }
      if (e.key === "Escape" && isSearchOpen) {
        setIsSearchOpen(false);
      }
    },
    [isSearchOpen]
  );

  // Listen for click events when the search modal is open
  useEffect(() => {
    if (isSearchOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isSearchOpen, handleClickOutside]);

  // Listen for keydown events
  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return (
    <>
      <button
        onClick={() => setIsSearchOpen(true)}
        className="w-[500px] flex cursor-pointer flex-row items-center group gap-x-4 h-10 rounded-xl justify-between pl-4 pr-2 text-sm dark:bg-zinc-300/5 bg-white/80 ring-1 backdrop-blur-sm dark:shadow-xl shadow-md shadow-zinc-300/15 hover:shadow-zinc-300/25 dark:shadow-zinc-950/15 dark:hover:shadow-zinc-950/20 ring-zinc-200 hover:ring-zinc-300 dark:ring-white/10 dark:hover:ring-white/15 transition-all ease-in-out duration-150 custom-tab-outline-offset-2"
      >
        <MagnifyingGlassIcon className="size-5 dark:text-white/80 text-zinc-500 group-hover:text-zinc-600 dark:group-hover:text-white/90 transition-all ease-in-out duration-150" />
        <p className="dark:text-white/70 text-zinc-500 group-hover:text-zinc-700 dark:group-hover:text-white/90 transition-all ease-in-out duration-150">
          Search documentation...
        </p>
        <p className="dark:text-white/70 text-zinc-500 group-hover:text-zinc-700 ring-1 dark:ring-white/5 ring-zinc-200 group-hover:ring-zinc-300 dark:group-hover:ring-white/10 dark:group-hover:text-white/90 ml-auto font-semibold text-xs bg-zinc-200/40 group-hover:bg-zinc-200/60 dark:bg-zinc-950/80 dark:group-hover:bg-zinc-950/90 transition-all ease-in-out duration-150 cursor-pointer px-2 py-1 rounded-lg whitespace-nowrap">
          Ctrl K
        </p>
      </button>

      <SearchPortal>
        <div
          className={clsx(
            "fixed inset-0 z-50",
            isSearchOpen ? "pointer-events-auto" : "pointer-events-none"
          )}
        >
          {/* Backdrop */}
          <div
            className={clsx(
              "fixed inset-0 h-screen w-full dark:bg-zinc-950/20 bg-zinc-950/5 backdrop-blur-xs transition-opacity duration-300 ease-in-out",
              isSearchOpen ? "opacity-100" : "opacity-0"
            )}
          />

          {/* Dialog */}
          <div
            ref={dialogRef}
            className={clsx(
              "dark:bg-zinc-900 bg-white fixed mx-auto inset-0 transition-all duration-200 ease-in-out h-fit mt-32 rounded-xl shadow-xl dark:shadow-zinc-950/30 shadow-zinc-500/10 ring-1 ring-zinc-200 dark:ring-white/15 w-[640px]",
              isSearchOpen
                ? "opacity-100 scale-100"
                : "opacity-0 scale-[0.97] translate-y-[2px]"
            )}
          >
            <div className="py-4 px-5 border-b-[1.5px] gap-x-6 dark:border-zinc-950 border-zinc-200 flex flex-row items-center justify-between">
              <input
                ref={inputRef}
                type="text"
                placeholder="What are you looking for?"
                className="w-full bg-transparent dark:text-white dark:placeholder:text-white/60 text-zinc-800 placeholder:text-zinc-500 outline-hidden"
              />
              <button
                onClick={() => setIsSearchOpen(false)}
                className="dark:text-white/70 text-zinc-500 transition-all custom-tab-outline-offset-2 hover:text-zinc-700 dark:bg-zinc-950/80 ring-1 ring-zinc-200 hover:ring-zinc-300 dark:ring-white/5 dark:hover:ring-white/10 dark:hover:text-white/90 bg-zinc-200/40 dark:hover:bg-zinc-950 hover:bg-zinc-200/60 ease-in-out duration-150 cursor-pointer px-2 py-1 rounded-md whitespace-nowrap font-semibold text-xs"
              >
                Esc
              </button>
            </div>
            <div className="border-t-[1.5px] dark:border-white/10 border-zinc-100 py-4 px-5">
              <div className="text-sm dark:text-white/60 text-zinc-500">
                No recent searches
              </div>
            </div>
          </div>
        </div>
      </SearchPortal>
    </>
  );
}
