"use client";

// React Imports
import { FC, useCallback, PropsWithChildren } from "react";

// Next Imports
import { useRouter } from "next/navigation";

// Types Imports
import { HeaderType } from "@/types/page-tree-element/Header";

// Components Imports
import { CopyLinkButton } from "./components/CopyLinkButton";

// Utils Imports
import clsx from "clsx";

// Interface for the Header component props
interface HeaderProps {
  id: string;
  Level: HeaderType;
  className: string;
}

// Styles for different header levels
const headerStyles = {
  h1: "text-4xl font-semibold tracking-tight dark:text-white text-zinc-800 text-start mt-4",
  h2: "text-3xl font-semibold tracking-tight dark:text-white text-zinc-800 text-start mt-3",
  h3: "text-2xl font-medium tracking-tight dark:text-white text-zinc-800 text-start mt-2",
  h4: "text-xl font-medium tracking-normal dark:text-white text-zinc-800 text-start mt-1",
  h5: "text-lg font-[450] tracking-normal dark:text-zinc-300 text-zinc-700 text-start mt-0.5",
  h6: "text-base font-[450] tracking-normal dark:text-zinc-300 text-zinc-700 text-start",
};

// Header component
export const Header: FC<PropsWithChildren<HeaderProps>> = ({
  id,
  Level,
  children,
  className,
}) => {
  const router = useRouter();

  // Clicking on a header title will scroll to it
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

  // Render the Header component
  return (
    <Level
      id={id}
      onClick={handleClick}
      className={clsx(
        headerStyles[Level],
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
};
