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
      className="dark:hover:text-white/80 cursor-pointer w-fit h-fit text-zinc-500 hover:text-zinc-400 dark:text-white/70 transition-all ease-in-out duration-150 rounded-xs custom-tab-outline-offset-4"
    >
      <Search className="size-5" />
    </button>
  );
}
