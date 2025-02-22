"use client";

// Heroicons Imports
import { LinkIcon } from "@heroicons/react/24/outline";

interface CopyLinkButtonProps {
  headerId: string;
}

export default function CopyLinkButton({ headerId }: CopyLinkButtonProps) {
  return (
    <div
      onClick={(e) => {
        // Don't trigger the click event of the parent element
        e.stopPropagation();
        // Copy the link to the element to the clipboard
        navigator.clipboard.writeText(
          `${window.location.origin}${window.location.pathname}#${headerId}`
        );
      }}
      className="flex items-center justify-center rounded-md size-6 dark:text-white/70 dark:hover:text-white/90 cursor-pointer text-zinc-500 hover:text-zinc-700 font-semibold text-xs bg-zinc-100/80 hover:bg-zinc-100 dark:bg-zinc-900/80 dark:hover:bg-zinc-900 ring-1 ring-zinc-200 hover:ring-zinc-300 dark:ring-white/10 dark:hover:ring-white/15 transition-all duration-100 ease-in-out whitespace-nowrap"
    >
      <LinkIcon className="size-4" />
    </div>
  );
}
