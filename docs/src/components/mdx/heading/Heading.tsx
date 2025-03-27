"use client";

// React Imports
import { useCallback } from "react";

// Next Imports
import { useRouter } from "next/navigation";

// Types Imports
import { HeadingTypes } from "@/types/page/Heading";

// Components Imports
import { CopyLinkButton } from "./components/CopyLinkButton";

// Utils Imports
import clsx from "clsx";

// Interface for the Heading component props
interface HeadingProps {
  id: string;
  Level: HeadingTypes;
  children: React.ReactNode;
  className: string;
}

// Styles for different heading levels
const headingStyles = {
  [HeadingTypes.H1]:
    "text-4xl font-semibold tracking-tight dark:text-white text-zinc-800 text-start mt-4",
  [HeadingTypes.H2]:
    "text-3xl font-semibold tracking-tight dark:text-white text-zinc-800 text-start mt-3",
  [HeadingTypes.H3]:
    "text-2xl font-medium tracking-tight dark:text-white text-zinc-800 text-start mt-2",
  [HeadingTypes.H4]:
    "text-xl font-medium tracking-normal dark:text-white text-zinc-800 text-start mt-1",
  [HeadingTypes.H5]:
    "text-lg font-[450] tracking-normal dark:text-zinc-300 text-zinc-700 text-start mt-0.5",
  [HeadingTypes.H6]:
    "text-base font-[450] tracking-normal dark:text-zinc-300 text-zinc-700 text-start",
};

// Heading component
export function Heading({
  id,
  Level,
  children,
  className,
}: HeadingProps) {
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

  // Render the Heading component
  return (
    <Level
      id={id}
      onClick={handleClick}
      className={clsx(
        headingStyles[Level],
        "group relative w-fit cursor-pointer scroll-mt-[94px] outline-hidden transition-colors duration-150 ease-in-out", // 94px scroll offset to account for the topbar
        className,
      )}
    >
      {children}
      <div className="absolute top-0 right-full bottom-0 hidden h-full items-center justify-center pr-3 opacity-0 transition-opacity duration-150 ease-in-out group-hover:opacity-100 md:flex">
        <CopyLinkButton headerId={id} />
      </div>
    </Level>
  );
}
