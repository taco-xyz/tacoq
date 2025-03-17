"use client";

// Heroicons Imports
import { Bars3CenterLeftIcon } from "@heroicons/react/24/outline";

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
      <Bars3CenterLeftIcon className="size-5" />
    </button>
  );
}
