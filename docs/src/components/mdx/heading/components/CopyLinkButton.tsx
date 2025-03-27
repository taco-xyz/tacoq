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
        `${window.location.origin}${window.location.pathname}#${headerId}`,
      );
      // Set the copied state to true and reset it after 500ms
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 800);
    },
    [headerId],
  );

  // Render the CopyLinkButton component
  return (
    <div
      onClick={handleCopyClick}
      className="flex size-6 flex-shrink-0 cursor-pointer items-center justify-center rounded-md bg-zinc-100/80 text-zinc-500 ring-1 ring-zinc-200 transition-all duration-100 ease-in-out hover:bg-zinc-100 hover:text-zinc-700 hover:ring-zinc-300 dark:bg-zinc-900/80 dark:text-white/70 dark:ring-white/10 dark:hover:bg-zinc-900 dark:hover:text-white/90 dark:hover:ring-white/15"
    >
      <Check
        className={clsx(
          "absolute size-4 transition-opacity duration-150 ease-in-out",
          isCopied ? "opacity-100" : "opacity-0",
        )}
      />
      <Link
        className={clsx(
          "absolute size-4 transition-opacity duration-150 ease-in-out",
          isCopied ? "opacity-0" : "opacity-100",
        )}
      />
    </div>
  );
}
