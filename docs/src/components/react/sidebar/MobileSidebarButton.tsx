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
      className="dark:hover:text-white/80 w-fit h-fit text-zinc-500 hover:text-zinc-400 cursor-pointer dark:text-white/70 transition-all ease-in-out duration-150 rounded-xs custom-tab-outline-offset-4"
    >
      <Text className="size-5" />
    </button>
  );
}
