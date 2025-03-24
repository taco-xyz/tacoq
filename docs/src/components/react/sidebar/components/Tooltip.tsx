"use client";

// Lucide Icons
import { ArrowUp, ArrowDown, ArrowLeft, ArrowRight } from "lucide-react";

// Tailwind Imports
import clsx from "clsx";

// Context Imports
import { useTooltip } from "@/components/react/sidebar/context/TooltipContext";

export default function Tooltip() {
  // Extract the tooltip context
  const {
    tooltipProps: {
      appearance: {
        topPosition,
        arrowPosition,
        leftPosition,
        visible,
        animationDirection,
      },
      content,
      previousContent,
    },
    contentContainerRef,
    currentContentContainerRef,
  } = useTooltip();

  return (
    <div
      className={clsx(
        "fixed dark:bg-zinc-900 z-50 bg-white rounded-md flex w-fit flex-col items-center justify-center min-w-72 ring-1 ring-zinc-200 dark:ring-white/10 shadow-sm shadow-zinc-900/10 dark:shadow-white/5",
        "transition-all duration-100 ease-in-out",
        visible
          ? "opacity-100 pointer-events-auto"
          : "opacity-0 pointer-events-none"
      )}
      style={{
        top: topPosition ?? undefined,
        left: leftPosition ?? undefined,
      }}
    >
      {/* Tooltip arrow */}
      <div
        className="absolute -left-[5.5px] dark:bg-zinc-900 bg-white rounded-[1.5px] rounded-r-none rounded-t-none size-2.5 rotate-45 origin-center border-l-[1.5px] border-b-[1.5px] border-zinc-200 dark:border-white/5"
        style={{
          top: arrowPosition === "top" ? "8px" : undefined,
          bottom: arrowPosition === "bottom" ? "8px" : undefined,
        }}
      />

      {/* Tooltip content */}
      <div className="flex flex-col w-full">
        {/* Page preview */}
        <div className="w-full px-2.5 justify-start gap-y-1 py-2 border-b-[1.5px] dark:border-zinc-950 border-zinc-200 overflow-hidden relative">
          {/* Content container with dynamic height */}
          <div
            ref={contentContainerRef}
            className="relative transition-all duration-150 ease-in-out w-full"
          >
            {/* Current content */}
            <div
              ref={currentContentContainerRef}
              className="absolute w-full h-fit"
              key={`current-content-${content.title}`} // Forces a re-render when the contentBuffer changes
            >
              <div
                className={clsx(
                  "flex flex-col w-full justify-start gap-y-1",
                  animationDirection === "down" && "animate-slide-in-up",
                  animationDirection === "up" && "animate-slide-in-down"
                )}
              >
                <span className="text-xs font-medium text-zinc-800 w-full dark:text-white whitespace-nowrap">
                  {content.title}
                </span>
                <span className="text-xs font-normal text-zinc-500 w-full dark:text-zinc-400">
                  {content.description}
                </span>
              </div>
            </div>
            {/* Previous content */}
            <div
              className="absolute w-full h-fit"
              key={`previous-content-${previousContent?.title || "undefined"}`} // Forces a re-render when the contentBuffer changes
            >
              <div
                className={clsx(
                  "flex flex-col w-full justify-start gap-y-1",
                  animationDirection === "down" && "animate-slide-out-up",
                  animationDirection === "up" && "animate-slide-out-down",
                  "opacity-0"
                )}
              >
                <span className="text-xs font-medium text-zinc-800 dark:text-white whitespace-nowrap">
                  {previousContent?.title}
                </span>
                <span className="text-xs font-normal text-zinc-500 dark:text-zinc-400">
                  {previousContent?.description}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* Hot-key Info */}
        <div className="flex flex-row items-center justify-start px-2.5 py-2.5 border-t-[1.5px] dark:border-white/10 border-zinc-100 w-full overflow-hidden">
          <div className="flex flex-row items-center gap-x-1.5 pr-10">
            <div className="dark:text-white/70 flex items-center justify-center text-zinc-500 dark:bg-zinc-950/70 ring-1 ring-zinc-200 dark:ring-white/5   bg-zinc-200/40 transition-all ease-in-out duration-150 cursor-pointer p-1 rounded-md">
              <ArrowUp className="size-3" />

              <ArrowDown className="size-3" />
            </div>
            <p className="text-xs font-normal text-zinc-500 dark:text-zinc-400 whitespace-nowrap">
              to navigate
            </p>
          </div>

          <div
            className={clsx(
              "transition-all duration-100 ease-in-out",
              content.isFolder ? "opacity-100" : "opacity-0"
            )}
          >
            <div
              className={clsx(
                "flex flex-row items-center gap-x-1.5 transition-all duration-200 ease-in-out",
                content.isFolder ? " w-[191px] pr-10" : "w-0"
              )}
            >
              <div className="dark:text-white/70 flex items-center justify-center text-zinc-500 dark:bg-zinc-950/70 ring-1 ring-zinc-200 dark:ring-white/5   bg-zinc-200/40 transition-all ease-in-out duration-150 cursor-pointer p-1 rounded-md whitespace-nowrap">
                <ArrowLeft className="size-3" />

                <ArrowRight className="size-3" />
              </div>
              <p className="text-xs font-normal text-zinc-500 dark:text-zinc-400 whitespace-nowrap">
                or
              </p>
              <div className="dark:text-white/70 flex items-center font-mono justify-center text-zinc-500 dark:bg-zinc-950/70 ring-1 ring-zinc-200 dark:ring-white/5   bg-zinc-200/40 transition-all ease-in-out duration-150 cursor-pointer py-0.5 px-1.5 rounded-md whitespace-nowrap font-medium text-xs">
                Space
              </div>
              <p className="text-xs font-normal text-zinc-500 dark:text-zinc-400 whitespace-nowrap">
                to open
              </p>
            </div>
          </div>

          <div
            className={clsx(
              "transition-all duration-100 ease-in-out",
              content.isUrl ? "opacity-100" : "opacity-0"
            )}
          >
            <div
              className={clsx(
                "flex flex-row items-center gap-x-1.5 transition-all duration-200 ease-in-out",
                content.isUrl ? "w-[142px] pr-10" : "w-0"
              )}
            >
              <div className="dark:text-white/70 flex items-center font-mono justify-center text-zinc-500 dark:bg-zinc-950/70 ring-1 ring-zinc-200 dark:ring-white/5   bg-zinc-200/40 transition-all ease-in-out duration-150 cursor-pointer py-0.5 px-1.5 rounded-md whitespace-nowrap font-medium text-xs">
                Enter
              </div>
              <p className="text-xs font-normal text-zinc-500 dark:text-zinc-400 whitespace-nowrap">
                to select
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
