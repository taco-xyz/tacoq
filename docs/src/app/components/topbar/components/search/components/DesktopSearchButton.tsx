"use client";

// Heroicons Imports
import { MagnifyingGlassIcon } from "@heroicons/react/24/outline";

// Context Imports
import { useSearch } from "../context/SearchContext";

export function DesktopSearchButton() {
  // Extract the Search Context
  const { openSearch } = useSearch();

  return (
    <button
      onClick={openSearch}
      className="w-full sm:flex hidden cursor-pointer flex-row items-center group gap-x-4 h-10 rounded-xl pl-4 pr-2 text-sm dark:bg-zinc-300/5 bg-white/80 ring-1 backdrop-blur-sm dark:shadow-xl shadow-md shadow-zinc-300/15 hover:shadow-zinc-300/25 dark:shadow-zinc-950/15 dark:hover:shadow-zinc-950/25 ring-zinc-200 hover:ring-zinc-300 dark:ring-white/10 dark:hover:ring-white/15 transition-all ease-in-out duration-150 custom-tab-outline-offset-2"
    >
      <MagnifyingGlassIcon className="size-5 dark:text-white/80 text-zinc-500 group-hover:text-zinc-600 dark:group-hover:text-white/90 transition-all ease-in-out duration-150" />
      <p className="dark:text-white/70 text-zinc-500 group-hover:text-zinc-700 dark:group-hover:text-white/90 transition-all ease-in-out duration-150">
        Search documentation...
      </p>
      <p className="dark:text-white/70 text-zinc-500 group-hover:text-zinc-700 ring-1 dark:ring-white/5 ring-zinc-200 group-hover:ring-zinc-300 dark:group-hover:ring-white/10 dark:group-hover:text-white/90 ml-auto font-semibold text-xs bg-zinc-200/40 group-hover:bg-zinc-200/60 dark:bg-zinc-950/80 dark:group-hover:bg-zinc-950/90 transition-all ease-in-out duration-150 cursor-pointer px-2 py-1 rounded-lg whitespace-nowrap">
        Ctrl K
      </p>
    </button>
  );
}
