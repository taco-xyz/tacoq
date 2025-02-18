"use client";

// React Imports
import { createPortal } from "react-dom";

// Next Imports
import dynamic from "next/dynamic";

// Utils Imports
import clsx from "clsx";

// Context Imports
import { useSearch } from "../context/SearchContext";

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
 * SearchDialog component that serves as the search modal for the application
 * Contains:
 * - Search input with autofocus
 * - Search results
 * - Search suggestions
 */
export function SearchDialog() {
  // Extract the Search Context
  const { isSearchOpen, closeSearch, dialogRef, inputRef } = useSearch();

  return (
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
            "dark:bg-zinc-900 bg-white fixed mx-auto inset-0 transition-all md:duration-200 duration-300 ease-in-out md:h-fit h-full md:mt-32 mt-50 md:rounded-xl rounded-t-xl shadow-xl dark:shadow-zinc-950/30 shadow-zinc-500/10 ring-1 ring-zinc-200 dark:ring-white/15 md:w-[640px] w-[95%]",
            isSearchOpen
              ? "opacity-100 scale-100 translate-y-0"
              : "opacity-0 md:scale-[0.97] scale-100 md:translate-y-[2px] translate-y-full"
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
              onClick={closeSearch}
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
  );
}
