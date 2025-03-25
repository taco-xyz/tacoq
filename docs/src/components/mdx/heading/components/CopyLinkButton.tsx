"use client";

// React Imports
import { useState, useCallback } from "react";

// Lucide Icons Imports
import { Link, Check } from "lucide-react";

// Utils Imports
import clsx from "clsx";

// Interface for the CopyLinkButton component props
interface CopyLinkButtonProps {
  headerId: string;
}

// CopyLinkButton component
export default function CopyLinkButton({ headerId }: CopyLinkButtonProps) {
  const [isCopied, setIsCopied] = useState(false);

  // Handle the click event on the CopyLinkButton
  const handleCopyClick = useCallback(
    (e: React.MouseEvent) => {
      // Don't trigger the click event of the parent element
      e.stopPropagation();
      // Copy the link to the element to the clipboard
      navigator.clipboard.writeText(
        `${window.location.origin}${window.location.pathname}#${headerId}`
      );
      // Set the copied state to true and reset it after 500ms
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 800);
    },
    [headerId]
  );

  // Render the CopyLinkButton component
  return (
    <div
      onClick={handleCopyClick}
      className="flex flex-shrink-0 items-center justify-center rounded-md size-6 dark:text-white/70 dark:hover:text-white/90 cursor-pointer text-zinc-500 hover:text-zinc-700 bg-zinc-100/80 hover:bg-zinc-100 dark:bg-zinc-900/80 dark:hover:bg-zinc-900 ring-1 ring-zinc-200 hover:ring-zinc-300 dark:ring-white/10 dark:hover:ring-white/15 transition-all duration-100 ease-in-out"
    >
      <Check
        className={clsx(
          "absolute size-4 transition-opacity duration-150 ease-in-out",
          isCopied ? "opacity-100" : "opacity-0"
        )}
      />
      <Link
        className={clsx(
          "absolute size-4 transition-opacity duration-150 ease-in-out",
          isCopied ? "opacity-0" : "opacity-100"
        )}
      />
    </div>
  );
}
