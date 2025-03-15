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
      <div className="flex flex-col w-full gap-y-10 px-8 md:py-20 py-10 relative overflow-hidden max-w-(--breakpoint-2xl) border-b border-zinc-200 dark:border-white/10">
        <div
          className={clsx(
            "flex flex-row w-full gap-x-20 justify-between items-center "
          )}
        >
          {/* Logo */}
          <Logo />

          {/* Social Links Desktop */}
          <div className="md:flex flex-row space-x-4 items-center hidden">
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
        </div>

        {/* Link Groups */}
        <div className="grid rounded-lg w-full grid-cols-[repeat(auto-fit,minmax(100px,1fr))] gap-y-8 gap-x-8 lg:max-w-xl xl:max-w-2xl mx-auto lg:-mt-16">
          {footerContent.linkGroups?.map((group, idx) => (
            <div
              key={idx}
              className="flex flex-col gap-y-3 whitespace-nowrap text-sm w-fit"
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

        {/* Social Links Mobile */}
        <div className="flex flex-row space-x-4 items-center md:hidden">
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
        <div className="bg-radial absolute origin-center left-1/2 -translate-x-1/2 -bottom-20 h-36 w-[100rem] opacity-10 dark:opacity-4 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
      </div>

      {/* Copyright */}
      <div className="flex flex-row items-center justify-center w-full max-w-(--breakpoint-2xl) py-8 px-8">
        <p className="text-zinc-400 dark:text-white/40 text-xs font-mono">
          &copy; {new Date().getFullYear()} Taco. All rights reserved.
        </p>
      </div>
    </div>
  );
}
