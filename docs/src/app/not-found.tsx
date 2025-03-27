// Next Imports
import Link from "next/link";

// Lucide Icons
import { ArrowLeft } from "lucide-react";

export default function NotFound() {
  return (
    <div className="flex flex-col items-start justify-start gap-y-6 border-b border-zinc-200 pb-9 transition-colors duration-150 ease-in-out dark:border-zinc-800">
      <div className="flex flex-col items-start justify-start gap-y-3">
        <div className="flex w-fit flex-row items-center justify-center gap-x-2 font-mono text-xs font-semibold text-zinc-500 uppercase transition-colors duration-150 ease-in-out dark:text-zinc-400">
          Error 404
        </div>

        <h1 className="text-4xl font-semibold tracking-tight text-zinc-800 transition-colors duration-150 ease-in-out dark:text-white">
          Page Not Found
        </h1>
      </div>

      <h5 className="text-lg font-[450] tracking-normal text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-300">
        We couldn&apos;t find the page you were looking for.
      </h5>

      <Link
        href=".."
        className="custom-tab-outline-offset-4 flex cursor-pointer flex-row items-center gap-x-2 rounded-lg px-1.5 py-1 text-zinc-600 transition-all duration-150 ease-in-out hover:bg-zinc-800/5 hover:text-zinc-800 dark:text-zinc-300 dark:hover:bg-white/5 hover:dark:text-white"
      >
        <ArrowLeft className="size-3.5" />
        <p className="text-sm font-medium">Go back</p>
      </Link>
    </div>
  );
}
