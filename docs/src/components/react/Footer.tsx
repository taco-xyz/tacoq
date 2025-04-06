"use client";

// React Imports
import { FC } from "react";

// Next Imports
import Link from "next/link";

// Components Imports
import { DivisionLogo } from "./Logo";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

// Utils Imports
import clsx from "clsx";
import { Status } from "@/types/FooterContent";

export const Footer: FC = () => {
  // Extract footer content from the context
  const { footerContent } = usePageTree();

  return (
    <div className="absolute flex w-full flex-col items-center justify-center border-t border-t-zinc-200 bg-white/50 transition-all duration-150 ease-in-out dark:border-t-white/10 dark:bg-zinc-950/50">
      <div className="relative flex w-full max-w-(--breakpoint-2xl) flex-col gap-y-10 overflow-hidden border-b border-zinc-200 px-8 py-10 transition-colors duration-150 ease-in-out md:py-20 dark:border-white/10">
        <div
          className={clsx(
            "flex w-full flex-col items-start justify-between gap-x-40 gap-y-10 lg:flex-row",
          )}
        >
          {/* Company Logo */}
          <DivisionLogo />

          {/* Link Groups */}
          <div className="grid w-full grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-x-20 gap-y-8 rounded-lg">
            {footerContent.linkGroups?.map((group, idx) => (
              <div
                key={idx}
                className="flex flex-col gap-y-3 text-sm whitespace-nowrap"
              >
                <h3 className="font-semibold text-zinc-800 transition-colors duration-150 ease-in-out dark:text-white">
                  {group.groupName}
                </h3>
                <div className="flex flex-col gap-y-2">
                  {group.links.map((link, linkIdx) =>
                    link.status === Status.COMPLETED ? (
                      <Link
                        key={linkIdx}
                        href={link.url}
                        className="custom-tab-outline-offset-2 w-fit rounded-sm text-zinc-700 transition-all duration-150 ease-in-out hover:text-zinc-800 dark:text-white/70 dark:hover:text-white/90"
                      >
                        {link.linkName}
                      </Link>
                    ) : (
                      <div
                        key={linkIdx}
                        className="flex w-fit flex-row items-center gap-x-2"
                      >
                        <p className="text-zinc-500 dark:text-white/40">
                          {link.linkName}
                        </p>
                        <span
                          className={clsx(
                            "rounded-md px-1.5 py-0.5 text-xs ring-1 ring-inset",
                            link.status === Status.WORK_IN_PROGRESS
                              ? "bg-yellow-400/10 text-yellow-900 ring-yellow-400/25 dark:text-yellow-100"
                              : "bg-blue-400/10 text-blue-900 ring-blue-400/30 dark:text-blue-200",
                          )}
                        >
                          {link.status}
                        </span>
                      </div>
                    ),
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Decorative Background gradient */}
        <div className="pointer-events-none absolute -bottom-20 left-1/2 h-36 w-[100rem] origin-center -translate-x-1/2 bg-radial from-zinc-400 from-0% via-zinc-400/50 via-15% to-transparent to-50% opacity-10 dark:from-white dark:via-white/50 dark:opacity-4" />
      </div>

      {/* Copyright */}
      <div className="flex w-full max-w-(--breakpoint-2xl) flex-row items-center justify-center px-8 py-8">
        <p className="text-center font-mono text-xs text-zinc-400 dark:text-white/40">
          &copy; {new Date().getFullYear()} Taco Division.
          <br className="block sm:hidden" /> All rights reserved.
        </p>
      </div>
    </div>
  );
};
