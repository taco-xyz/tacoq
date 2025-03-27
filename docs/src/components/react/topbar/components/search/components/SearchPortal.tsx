"use client";

// React Imports
import { createPortal } from "react-dom";

// Next Imports
import dynamic from "next/dynamic";

// Lucide Icons
import { X } from "lucide-react";

// Utils Imports
import clsx from "clsx";

// Context Imports
import { useSearchModal } from "../context/SearchModalContext";

// Dynamic Components Imports
const SearchPortal = dynamic<{ children: React.ReactNode }>(
  () =>
    Promise.resolve(({ children }: { children: React.ReactNode }) =>
      createPortal(children, document.body),
    ),
  {
    ssr: false,
  },
);

/**
 * SearchDialog component that serves as the search modal for the application
 * Contains:
 * - Search input with autofocus
 * - Search results
 * - Search suggestions
 */
export function SearchDialog() {
  // Extract the Search Context
  const { isSearchOpen, closeSearch, dialogRef, inputRef } = useSearchModal();

  return (
    <SearchPortal>
      <div
        className={clsx(
          "fixed inset-0 z-50",
          isSearchOpen ? "pointer-events-auto" : "pointer-events-none",
        )}
      >
        {/* Backdrop */}
        <div
          className={clsx(
            "fixed inset-0 h-screen w-full bg-zinc-950/5 backdrop-blur-xs transition-opacity duration-300 ease-in-out dark:bg-zinc-950/20",
            isSearchOpen ? "opacity-100" : "opacity-0",
          )}
        />

        {/* Dialog */}
        <div
          ref={dialogRef}
          className={clsx(
            "fixed inset-0 mx-auto mt-50 h-full w-[95%] rounded-t-xl bg-white shadow-xl ring-1 shadow-zinc-500/10 ring-zinc-200 transition-all duration-300 ease-in-out md:mt-32 md:h-fit md:w-[640px] md:rounded-xl md:duration-200 dark:bg-zinc-900 dark:shadow-zinc-950/30 dark:ring-white/15",
            isSearchOpen
              ? "translate-y-0 scale-100 opacity-100"
              : "translate-y-full scale-100 opacity-0 md:translate-y-[2px] md:scale-[0.97]",
          )}
        >
          <div className="flex flex-row items-center justify-between gap-x-6 px-5 py-4">
            <input
              ref={inputRef}
              type="text"
              placeholder="What are you looking for?"
              className="w-full bg-transparent text-zinc-800 outline-hidden placeholder:text-zinc-500 dark:text-white dark:placeholder:text-white/60"
            />
            {/* Desktop Close Button */}
            <button
              onClick={closeSearch}
              className="custom-tab-outline-offset-2 hidden cursor-pointer rounded-md bg-zinc-200/40 px-2 py-1 font-mono text-xs font-semibold whitespace-nowrap text-zinc-500 ring-1 ring-zinc-200 transition-all duration-150 ease-in-out hover:bg-zinc-200/60 hover:text-zinc-700 hover:ring-zinc-300 md:block dark:bg-zinc-950/80 dark:text-white/70 dark:ring-white/5 dark:hover:bg-zinc-950 dark:hover:text-white/90 dark:hover:ring-white/10"
            >
              <p>Esc</p>
            </button>

            {/* Mobile Close Button */}
            <button
              onClick={closeSearch}
              className="custom-tab-outline-offset-4 cursor-pointer rounded-xs text-zinc-500 transition-all duration-150 ease-in-out hover:text-zinc-400 md:hidden dark:text-white/70 dark:hover:text-white/80"
            >
              <X className="size-6" />
            </button>
          </div>
          <div className="border-t-[1.5px] border-zinc-200 px-5 py-4 dark:border-white/10">
            <div className="text-sm text-zinc-500 dark:text-white/60">
              No recent searches
            </div>
          </div>
        </div>
      </div>
    </SearchPortal>
  );
}
