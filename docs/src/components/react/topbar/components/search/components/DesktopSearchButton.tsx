"use client";

// Lucide Icons
import { Search } from "lucide-react";

// Context Imports
import { useSearchModal } from "../context/SearchModalContext";
import { usePlatform } from "@/contexts/PlatformContext";
export function DesktopSearchButton() {
  // Extract the Search Context
  const { openSearch } = useSearchModal();

  // Extract the platform context
  const { isMacOS } = usePlatform();

  return (
    <button
      onClick={openSearch}
      className="group custom-tab-outline-offset-2 hidden h-10 w-full cursor-pointer flex-row items-center gap-x-4 rounded-xl bg-white/80 pr-2 pl-4 text-sm shadow-md ring-1 shadow-zinc-300/15 ring-zinc-200 backdrop-blur-sm transition-all duration-150 ease-in-out hover:shadow-zinc-300/25 hover:ring-zinc-300 sm:flex dark:bg-zinc-300/5 dark:shadow-xl dark:shadow-zinc-950/15 dark:ring-white/10 dark:hover:shadow-zinc-950/25 dark:hover:ring-white/15"
    >
      <Search className="size-5 text-zinc-500 transition-all duration-150 ease-in-out group-hover:text-zinc-600 dark:text-white/80 dark:group-hover:text-white/90" />
      <p className="text-zinc-500 transition-all duration-150 ease-in-out group-hover:text-zinc-700 dark:text-white/70 dark:group-hover:text-white/90">
        Search documentation...
      </p>
      <p className="ml-auto cursor-pointer rounded-lg bg-zinc-200/40 px-2 py-1 font-mono text-xs font-semibold whitespace-nowrap text-zinc-500 ring-1 ring-zinc-200 transition-all duration-150 ease-in-out group-hover:bg-zinc-200/60 group-hover:text-zinc-700 group-hover:ring-zinc-300 dark:bg-zinc-950/80 dark:text-white/70 dark:ring-white/5 dark:group-hover:bg-zinc-950/90 dark:group-hover:text-white/90 dark:group-hover:ring-white/10">
        {isMacOS ? "⌘" : "Ctrl"} K
      </p>
    </button>
  );
}
