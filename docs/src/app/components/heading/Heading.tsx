"use client";

// React Imports
import { useCallback } from "react";

// Next Imports
import { useRouter } from "next/navigation";

// Types Imports
import { HeadingTypes } from "@/types/page/Heading";

// Components Imports
import CopyLinkButton from "./components/CopyLinkButton";

// Utils Imports
import clsx from "clsx";

interface HeadingProps {
  id: string;
  Level: HeadingTypes;
  children: React.ReactNode;
  className: string;
}

const headingStyles = {
  [HeadingTypes.H1]:
    "text-4xl font-semibold tracking-tight dark:text-white text-zinc-700 text-start",
  [HeadingTypes.H2]:
    "text-3xl font-semibold tracking-tight dark:text-white text-zinc-700 text-start",
  [HeadingTypes.H3]:
    "text-2xl font-medium tracking-tight dark:text-white text-zinc-700 text-start",
  [HeadingTypes.H4]:
    "text-xl font-medium tracking-normal dark:text-white text-zinc-700 text-start",
  [HeadingTypes.H5]:
    "text-lg font-[450] tracking-normal dark:text-zinc-300 text-zinc-600 text-start",
  [HeadingTypes.H6]:
    "text-base font-[450] tracking-normal dark:text-zinc-300 text-zinc-600 text-start",
};

export default function Heading({ id, Level, children, className }: HeadingProps) {
  const router = useRouter();

  // Clicking on a heading title will scroll to it
  const handleClick = useCallback(() => {
    // Get the document element
    const element = document.getElementById(id);

    if (element) {
      // Scroll to the element
      element.scrollIntoView({ behavior: "smooth" });

      // Update URL without triggering a scroll
      router.push(`#${id}`, { scroll: false });
    }
  }, [id, router]);

  return (
    <button
      onClick={handleClick}
      // Don't focus this element on tab navigation
      tabIndex={-1}
      className="relative group flex flex-row items-center w-fit cursor-pointer outline-hidden"
    >
      <Level
        id={id}
        className={clsx(
          headingStyles[Level],
          "transition-colors duration-150 ease-in-out scroll-mt-[94px]", // 94px scroll offset to account for the topbar
          className
        )}
      >
        {children}
      </Level>
      <div className="absolute md:block hidden right-full opacity-0 group-hover:opacity-100 transition-opacity duration-150 ease-in-out pr-3">
        <CopyLinkButton headerId={id} />
      </div>
    </button>
  );
}
