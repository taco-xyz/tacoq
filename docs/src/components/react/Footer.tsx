"use client";

// Next Imports
import Link from "next/link";

// Components Imports
import { DivisionLogo } from "./Logo";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

// Utils Imports
import clsx from "clsx";
import { Status } from "@/types/FooterContent";

export function Footer() {
  // Extract footer content from the context
  const { footerContent } = usePageTree();

  return (
    <div className="items-center justify-center absolute flex flex-col w-full dark:border-t-white/10 border-t-zinc-200 border-t dark:bg-zinc-950/50 bg-white/50 transition-all ease-in-out duration-150">
      <div className="flex flex-col w-full gap-y-10 px-8 md:py-20 py-10 relative overflow-hidden max-w-(--breakpoint-2xl) border-b border-zinc-200 dark:border-white/10 transition-colors duration-150 ease-in-out">
        <div
          className={clsx(
            "flex lg:flex-row flex-col w-full gap-x-40 gap-y-10 justify-between items-start"
          )}
        >
          {/* Company Logo */}
          <DivisionLogo />

          {/* Link Groups */}
          <div className="grid rounded-lg w-full grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-y-8 gap-x-20">
            {footerContent.linkGroups?.map((group, idx) => (
              <div
                key={idx}
                className="flex flex-col gap-y-3 whitespace-nowrap text-sm"
              >
                <h3 className="font-semibold text-zinc-800 dark:text-white transition-colors ease-in-out duration-150">
                  {group.groupName}
                </h3>
                <div className="flex flex-col gap-y-2">
                  {group.links.map((link, linkIdx) =>
                    link.status === Status.COMPLETED ? (
                      <Link
                        key={linkIdx}
                        href={link.url}
                        className="text-zinc-700 w-fit dark:text-white/70 hover:text-zinc-800 dark:hover:text-white/90 transition-all ease-in-out duration-150 custom-tab-outline-offset-2 rounded-sm"
                      >
                        {link.linkName}
                      </Link>
                    ) : (
                      <div
                        key={linkIdx}
                        className="flex flex-row items-center gap-x-2 w-fit"
                      >
                        <p className="text-zinc-500 dark:text-white/40">
                          {link.linkName}
                        </p>
                        <span className={clsx("text-xs px-1.5 py-0.5 rounded-md ring-1 ring-inset", link.status === Status.WORK_IN_PROGRESS ? "dark:text-yellow-100 text-yellow-900 ring-yellow-400/25 bg-yellow-400/10" : "ring-blue-400/30 bg-blue-400/10 dark:text-blue-200 text-blue-900")}>
                          {link.status}
                        </span>
                      </div>
                    )
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Decorative Background gradient */}
        <div className="bg-radial absolute origin-center left-1/2 -translate-x-1/2 -bottom-20 h-36 w-[100rem] opacity-10 dark:opacity-4 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
      </div>

      {/* Copyright */}
      <div className="flex flex-row items-center justify-center w-full max-w-(--breakpoint-2xl) py-8 px-8">
        <p className="text-zinc-400 dark:text-white/40 text-xs font-mono">
          &copy; {new Date().getFullYear()} Taco Division. All rights reserved.
        </p>
      </div>
    </div>
  );
}
