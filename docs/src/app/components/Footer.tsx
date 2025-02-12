"use client";

// Next Imports
import Link from "next/link";

// Components Imports
import Logo from "./Logo";

// Context Imports
import { usePageTree } from "@/contexts/PageTreeContext";

// Utils Imports
import clsx from "clsx";

export function Footer() {
  // Extract footer content from the context
  const { footerContent } = usePageTree();

  return (
    <div className="items-center justify-center absolute flex flex-col w-full dark:border-t-white/10 border-t-zinc-200 border-t dark:bg-zinc-950/50 bg-white/50 transition-all ease-in-out duration-150">
      <div
        className={clsx(
          "flex flex-row relative overflow-hidden w-full max-w-(--breakpoint-2xl) px-8 gap-x-20 justify-between border-b border-zinc-200 dark:border-white/10",
          footerContent.linkGroups.length
            ? "items-start py-20"
            : "items-center py-10"
        )}
      >
        {/* Logo */}
        <div className="w-23">
          <Logo />
        </div>

        {/* Link Groups */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-y-8 gap-x-20">
          {footerContent.linkGroups?.map((group, idx) => (
            <div
              key={idx}
              className="flex flex-col gap-y-3 whitespace-nowrap text-sm"
            >
              <h3 className="font-medium text-zinc-700 dark:text-white transition-colors ease-in-out duration-150">
                {group.groupName}
              </h3>
              <div className="flex flex-col gap-y-2">
                {group.links.map((link, linkIdx) => (
                  <Link
                    key={linkIdx}
                    href={link.url}
                    className="text-zinc-500 dark:text-white/50 hover:text-zinc-700 dark:hover:text-white/70 transition-all ease-in-out duration-150 custom-tab-outline-offset-2 rounded-sm"
                  >
                    {link.linkName}
                  </Link>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Social Links */}
        <div className="flex flex-row space-x-4 w-23 items-center justify-end">
          {footerContent.socialLinks?.map((social, idx) => (
            <Link
              key={idx}
              href={social.url}
              className="dark:hover:text-white/80 text-zinc-500 hover:text-zinc-400 dark:text-white/70 transition-all ease-in-out duration-150 rounded-xs custom-tab-outline-offset-4"
            >
              <social.Icon className="size-5" />
            </Link>
          ))}
        </div>

        {/* Decorative Background gradient */}
        <div className="bg-radial absolute origin-center -bottom-20 h-36 w-[100rem] opacity-10 dark:opacity-4 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
      </div>

      {/* Copyright */}
      <div className="flex flex-row items-center justify-center w-full max-w-(--breakpoint-2xl) py-8 px-8">
        <p className="text-zinc-400 dark:text-white/40 text-xs">
          &copy; {new Date().getFullYear()} Taco. All rights reserved.
        </p>
      </div>
    </div>
  );
}
