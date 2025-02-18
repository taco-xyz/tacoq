"use client";

// Heroicons Imports
import { MagnifyingGlassIcon } from "@heroicons/react/24/outline";

// Context Imports
import { useSearch } from "../context/SearchContext";

export function MobileSearchButton() {
  // Extract the Search Context
  const { openSearch } = useSearch();

  return (
    <button
      onClick={openSearch}
      className="dark:hover:text-white/80 w-fit h-fit text-zinc-500 hover:text-zinc-400 dark:text-white/70 transition-all ease-in-out duration-150 rounded-xs custom-tab-outline-offset-4"
    >
      <MagnifyingGlassIcon className="size-5" />
    </button>
  );
}
