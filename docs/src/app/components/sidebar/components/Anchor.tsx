// Next Imports
import Link from "next/link";

// Types Imports
import type { Anchor } from "@/types/Anchor";

export default function AnchorComponent({ url, title, Icon }: Anchor) {
  return (
    <Link
      href={url}
      className="flex items-center gap-x-3.5 group w-full select-none rounded-md transition-all duration-150 ease-in-out custom-tab-outline-offset-4"
    >
      {Icon && (
        <Icon className="p-1 size-6 bg-white dark:bg-white/10 rounded-md shadow-xs shadow-zinc-900/20 dark:shadow-white/15 dark:group-hover:shadow-white/25 text-zinc-400 dark:text-white/50 dark:group-hover:text-white/70 dark:ring-white/15 dark:group-hover:ring-white/20 ring-1  ring-zinc-200 group-hover:ring-zinc-300 group-hover:text-zinc-500 transition-all duration-150 ease-in-out" />
      )}
      <p className="text-sm font-medium text-zinc-600 dark:text-zinc-300 group-hover:text-zinc-900 dark:group-hover:text-zinc-100 transition-all duration-150 ease-in-out">
        {title}
      </p>
    </Link>
  );
}
