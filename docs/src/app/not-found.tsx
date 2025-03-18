"use client";

// Next Imports
import { useRouter } from "next/navigation";

// Heroicons
import { ArrowLeftIcon } from "@heroicons/react/24/outline";

export default function NotFound() {
  const router = useRouter();

  return (
    <div className="flex flex-col items-start justify-start gap-y-6 border-b border-zinc-200 dark:border-zinc-800 pb-9 transition-colors duration-150 ease-in-out">
      <div className="flex flex-col items-start justify-start gap-y-3">
        <div className="flex flex-row items-center w-fit justify-center gap-x-2 font-mono uppercase text-xs font-semibold text-zinc-500 dark:text-zinc-400 transition-colors duration-150 ease-in-out">
          Error 404
        </div>

        <h1 className="text-4xl font-semibold tracking-tight dark:text-white text-zinc-800 transition-colors duration-150 ease-in-out">
          Page Not Found
        </h1>
      </div>

      <h5 className="text-lg font-[450] tracking-normal dark:text-zinc-300 text-zinc-600 transition-colors duration-150 ease-in-out">
        We couldn&apos;t find the page you were looking for.
      </h5>

      <button
        onClick={() => router.back()}
        className=" text-zinc-600 hover:bg-zinc-800/5 dark:hover:bg-white/5 hover:dark:text-white hover:text-zinc-800 py-1 px-1.5 dark:text-zinc-300 transition-all duration-150 ease-in-out flex flex-row items-center gap-x-2 rounded-lg custom-tab-outline-offset-4 cursor-pointer"
      >
        <ArrowLeftIcon className="size-3.5" />{" "}
        <p className="text-sm font-medium">Go back</p>
      </button>
    </div>
  );
}
