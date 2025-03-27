// Next Imports
import Link from "next/link";

// Types Imports
import type { Anchor } from "@/types/Anchor";

export default function AnchorComponent({ url, title, Icon }: Anchor) {
  return (
    <Link
      href={url}
      className="group custom-tab-outline-offset-4 flex w-full items-center gap-x-3.5 rounded-md transition-all duration-150 ease-in-out select-none"
    >
      {Icon && (
        <Icon className="size-6.5 rounded-md bg-white p-[4.5px] text-zinc-400 shadow-xs ring-1 shadow-zinc-900/20 ring-zinc-200 transition-all duration-150 ease-in-out ring-inset group-hover:text-zinc-500 group-hover:ring-zinc-300 dark:bg-white/10 dark:text-white/50 dark:shadow-white/15 dark:ring-white/15 dark:group-hover:text-white/70 dark:group-hover:shadow-white/25 dark:group-hover:ring-white/20" />
      )}
      <p className="text-sm font-medium text-zinc-600 transition-all duration-150 ease-in-out group-hover:text-zinc-900 dark:text-zinc-300 dark:group-hover:text-zinc-100">
        {title}
      </p>
    </Link>
  );
}
