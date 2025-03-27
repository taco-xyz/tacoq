"use client";

// Lucide Icons
import { Search } from "lucide-react";

// Context Imports
import { useSearchModal } from "../context/SearchModalContext";

export default function MobileSearchButton() {
  // Extract the Search Context
  const { openSearch } = useSearchModal();

  return (
    <button
      onClick={openSearch}
      className="custom-tab-outline-offset-4 h-fit w-fit cursor-pointer rounded-xs text-zinc-500 transition-all duration-150 ease-in-out hover:text-zinc-400 dark:text-white/70 dark:hover:text-white/80"
    >
      <Search className="size-5" />
    </button>
  );
}
