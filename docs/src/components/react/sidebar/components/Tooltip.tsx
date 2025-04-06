"use client";

// React Imports
import { FC } from "react";

// Lucide Icons
import { ArrowUp, ArrowDown, ArrowLeft, ArrowRight } from "lucide-react";

// Tailwind Imports
import clsx from "clsx";

// Context Imports
import { useTooltip } from "@/components/react/sidebar/context/TooltipContext";

export const Tooltip: FC = () => {
  // Extract the tooltip context
  const {
    tooltipProps: {
      appearance: {
        topPosition,
        arrowPosition,
        leftPosition,
        opacity,
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
        "fixed z-50 flex w-fit min-w-72 flex-col items-center justify-center rounded-md bg-white shadow-sm ring-1 shadow-zinc-900/10 ring-zinc-200 dark:bg-zinc-900 dark:shadow-white/5 dark:ring-white/10",
        "transition-all duration-100 ease-in-out",
        {
          "pointer-events-auto opacity-100": opacity === 100,
          "pointer-events-auto opacity-50": opacity === 50,
          "pointer-events-none opacity-0": opacity === 0,
        },
      )}
      style={{
        top: topPosition ?? undefined,
        left: leftPosition ?? undefined,
      }}
    >
      {/* Tooltip arrow */}
      <div
        className="absolute -left-[5.5px] size-2.5 origin-center rotate-45 rounded-[1.5px] rounded-t-none rounded-r-none border-b-[1.5px] border-l-[1.5px] border-zinc-200 bg-white dark:border-white/5 dark:bg-zinc-900"
        style={{
          top: arrowPosition === "top" ? "8px" : undefined,
          bottom: arrowPosition === "bottom" ? "8px" : undefined,
        }}
      />

      {/* Tooltip content */}
      <div className="flex w-full flex-col">
        {/* Page preview */}
        <div className="relative w-full justify-start gap-y-1 overflow-hidden px-2.5 py-2">
          {/* Content container with dynamic height */}
          <div
            ref={contentContainerRef}
            className="relative w-full transition-all duration-150 ease-in-out"
          >
            {/* Current content */}
            <div
              ref={currentContentContainerRef}
              className="absolute h-fit w-full"
              key={`current-content-${content.title}`} // Forces a re-render when the contentBuffer changes
            >
              <div
                className={clsx(
                  "flex w-full flex-col justify-start gap-y-1",
                  animationDirection === "down" && "animate-slide-in-up",
                  animationDirection === "up" && "animate-slide-in-down",
                )}
              >
                <span className="w-full text-xs font-medium whitespace-nowrap text-zinc-800 dark:text-white">
                  {content.title}
                </span>
                <span className="w-full text-xs font-normal text-zinc-500 dark:text-zinc-400">
                  {content.description}
                </span>
              </div>
            </div>
            {/* Previous content */}
            <div
              className="absolute h-fit w-full"
              key={`previous-content-${previousContent?.title || "undefined"}`} // Forces a re-render when the contentBuffer changes
            >
              <div
                className={clsx(
                  "flex w-full flex-col justify-start gap-y-1",
                  animationDirection === "down" && "animate-slide-out-up",
                  animationDirection === "up" && "animate-slide-out-down",
                  "opacity-0",
                )}
              >
                <span className="text-xs font-medium whitespace-nowrap text-zinc-800 dark:text-white">
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
        <div className="flex w-full flex-row items-center justify-start overflow-hidden border-t-[1.5px] border-zinc-200 px-2.5 py-2.5 dark:border-white/10">
          <div className="flex flex-row items-center gap-x-1.5 pr-10">
            <div className="flex cursor-pointer items-center justify-center rounded-md bg-zinc-200/40 p-1 text-zinc-500 ring-1 ring-zinc-200 transition-all duration-150 ease-in-out dark:bg-zinc-950/70 dark:text-white/70 dark:ring-white/5">
              <ArrowUp className="size-3" />

              <ArrowDown className="size-3" />
            </div>
            <p className="text-xs font-normal whitespace-nowrap text-zinc-500 dark:text-zinc-400">
              to navigate
            </p>
          </div>

          <div
            className={clsx(
              "transition-all duration-100 ease-in-out",
              content.isFolder ? "opacity-100" : "opacity-0",
            )}
          >
            <div
              className={clsx(
                "flex flex-row items-center gap-x-1.5 transition-all duration-200 ease-in-out",
                content.isFolder ? "w-[191px] pr-10" : "w-0",
              )}
            >
              <div className="flex cursor-pointer items-center justify-center rounded-md bg-zinc-200/40 p-1 whitespace-nowrap text-zinc-500 ring-1 ring-zinc-200 transition-all duration-150 ease-in-out dark:bg-zinc-950/70 dark:text-white/70 dark:ring-white/5">
                <ArrowLeft className="size-3" />

                <ArrowRight className="size-3" />
              </div>
              <p className="text-xs font-normal whitespace-nowrap text-zinc-500 dark:text-zinc-400">
                or
              </p>
              <div className="flex cursor-pointer items-center justify-center rounded-md bg-zinc-200/40 px-1.5 py-0.5 font-mono text-xs font-medium whitespace-nowrap text-zinc-500 ring-1 ring-zinc-200 transition-all duration-150 ease-in-out dark:bg-zinc-950/70 dark:text-white/70 dark:ring-white/5">
                Space
              </div>
              <p className="text-xs font-normal whitespace-nowrap text-zinc-500 dark:text-zinc-400">
                to open
              </p>
            </div>
          </div>

          <div
            className={clsx(
              "transition-all duration-100 ease-in-out",
              !content.isFolder ? "opacity-100" : "opacity-0",
            )}
          >
            <div
              className={clsx(
                "flex flex-row items-center gap-x-1.5 transition-all duration-200 ease-in-out",
                !content.isFolder ? "w-[142px] pr-10" : "w-0",
              )}
            >
              <div className="flex cursor-pointer items-center justify-center rounded-md bg-zinc-200/40 px-1.5 py-0.5 font-mono text-xs font-medium whitespace-nowrap text-zinc-500 ring-1 ring-zinc-200 transition-all duration-150 ease-in-out dark:bg-zinc-950/70 dark:text-white/70 dark:ring-white/5">
                Enter
              </div>
              <p className="text-xs font-normal whitespace-nowrap text-zinc-500 dark:text-zinc-400">
                to select
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
