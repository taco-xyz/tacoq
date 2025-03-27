"use client";

// Lucide Icons
import { Text } from "lucide-react";

// Context Imports
import { useMobileSidebarModal } from "./context/MobileSidebarModalContext";

export default function MobileSidebarButton() {
  // Extract the mobile sidebar context
  const { openSidebar } = useMobileSidebarModal();

  return (
    <button
      onClick={openSidebar}
      className="custom-tab-outline-offset-4 h-fit w-fit cursor-pointer rounded-xs text-zinc-500 transition-all duration-150 ease-in-out hover:text-zinc-400 dark:text-white/70 dark:hover:text-white/80"
    >
      <Text className="size-5" />
    </button>
  );
}
