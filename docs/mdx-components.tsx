// Next Imports
import Link from "next/link";
import Image from "next/image";

// Types Imports
import type { MDXComponents } from "mdx/types";
import { HeadingTypes, getHeaderId } from "@/types/page/Heading";
import { Card, CardGroup } from "@/components/mdx/Card";

// Components Imports
import Heading from "@/components/mdx/heading/Heading";
import Blockquote, {
  Important,
  Note,
  Tip,
  Warning,
  Caution,
} from "@/components/mdx/Blockquotes";

// Utils Imports
import clsx from "clsx";

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    ...components,

    // HEADINGS ------------------------------------------------------------
    h1: ({ children, className }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H1, name: children })}
        Level={HeadingTypes.H1}
        className={className}
      >
        {children}
      </Heading>
    ),

    h2: ({ children, className }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H2, name: children })}
        Level={HeadingTypes.H2}
        className={className}
      >
        {children}
      </Heading>
    ),

    h3: ({ children, className }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H3, name: children })}
        Level={HeadingTypes.H3}
        className={className}
      >
        {children}
      </Heading>
    ),

    h4: ({ children, className }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H4, name: children })}
        Level={HeadingTypes.H4}
        className={className}
      >
        {children}
      </Heading>
    ),

    h5: ({ children, className }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H5, name: children })}
        Level={HeadingTypes.H5}
        className={className}
      >
        {children}
      </Heading>
    ),

    h6: ({ children, className }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H6, name: children })}
        Level={HeadingTypes.H6}
        className={className}
      >
        {children}
      </Heading>
    ),

    // Cards ---------------------------------------------------

    Card: ({ className, ...props }) => (
      <Card className={className} {...props}></Card>
    ),

    CardGroup: ({ className, ...props }) => (
      <CardGroup className={className} {...props}></CardGroup>
    ),

    // HORIZONTAL RULE -----------------------------------------------------
    hr: ({ className }) => (
      <hr
        className={clsx(
          "border-t border-zinc-200 dark:border-zinc-800 my-6 transition-colors duration-150 ease-in-out",
          className
        )}
      />
    ),

    // IMAGE ---------------------------------------------------------------
    img: ({ src, alt, className }) => (
      <span className="w-full h-auto aspect-video ring-inset flex my-2 ring-1 rounded-2xl p-1.5  ring-zinc-200 dark:ring-zinc-800/70 shadow-xl shadow-zinc-700/3 dark:shadow-black/5 transition-all duration-150 ease-in-out">
        <span className="w-full h-full relative">
          <Image
            src={src}
            alt={alt}
            fill
            className={clsx(
              "rounded-[13px] object-cover object-center",
              className
            )}
            quality={100}
          />
        </span>
      </span>
    ),

    // CODEBLOCK WRAPPER --------------------------------------------------
    pre: ({ children, className }) => (
      <div className="w-full h-fit p-1.5 ring-1 rounded-2xl ring-inset ring-zinc-200 dark:ring-zinc-800/70 flex items-center shadow-xl shadow-zinc-700/3 dark:shadow-black/5 justify-center transition-all duration-150 ease-in-out">
        <pre
          className={clsx(
            className,
            "bg-zinc-700 w-full dark:bg-zinc-900 ring-1 ring-zinc-300 dark:ring-zinc-800 overflow-x-auto rounded-[11px] transition-all duration-150 ease-in-out shadow-2xl shadow-zinc-700/3 dark:shadow-black/5"
          )}
        >
          {children}
        </pre>
      </div>
    ),

    // BLOCKQUOTE ----------------------------------------------------------
    blockquote: ({ children, className }) => (
      <Blockquote className={className}>{children}</Blockquote>
    ),

    // CUSTOM BLOCKQUOTE CALLOUTS -------------------------------------------
    Note: ({ children, className }) => (
      <Note className={className}>{children}</Note>
    ),

    Tip: ({ children, className }) => (
      <Tip className={className}>{children}</Tip>
    ),

    Important: ({ children, className }) => (
      <Important className={className}>{children}</Important>
    ),

    Warning: ({ children, className }) => (
      <Warning className={className}>{children}</Warning>
    ),

    Caution: ({ children, className }) => (
      <Caution className={className}>{children}</Caution>
    ),

    // QUOTABLE COMPONENTS -------------------------------------------------

    // QUOTABLE AS INLINE CODE
    code: ({ children, className }) => (
      <code
        className={clsx(
          "font-mono text-sm text-[#ce9178] font-medium transition-all duration-150 ease-in-out",
          // Code block
          "[pre_&]:py-4 [pre_&]:px-0 [pre_&]:bg-transparent",
          // Inline code
          "py-0.5 px-1.5 rounded-[5px] bg-zinc-100 dark:bg-zinc-800 [&:not(pre_code)]:mx-1 [&:not(pre_code)]:whitespace-nowrap [blockquote_&]:text-inherit [blockquote_&]:bg-inherit [blockquote_&]:ring-1 [blockquote_&]:ring-inherit [blockquote_&]:ring-inset",
          className
        )}
      >
        {children}
      </code>
    ),

    p: ({ children, className }) => (
      <p
        className={clsx(
          "text-base tracking-normal dark:text-zinc-400 text-zinc-600 font-normal transition-colors duration-150 ease-in-out",
          "[blockquote_&]:text-inherit",
          className
        )}
      >
        {children}
      </p>
    ),

    a: ({ children, className, href }) => (
      <Link
        href={href}
        className={clsx(
          "dark:text-white text-zinc-800 font-semibold border-b dark:border-blue-400 border-blue-500 hover:border-b-[2px] transition-colors ease-in-out duration-150",
          "[blockquote_&]:text-inherit [blockquote_&]:border-inherit",
          className
        )}
      >
        {children}
      </Link>
    ),

    strong: ({ children, className }) => (
      <strong
        className={clsx(
          "font-semibold text-zinc-800 dark:text-white transition-colors duration-150 ease-in-out",
          "[blockquote_&]:text-inherit",
          className
        )}
      >
        {children}
      </strong>
    ),

    em: ({ children, className }) => (
      <em
        className={clsx(
          "italic text-zinc-800 dark:text-white transition-colors duration-150 ease-in-out",
          "[blockquote_&]:text-inherit",
          className
        )}
      >
        {children}
      </em>
    ),

    ol: ({ children, className }) => (
      <ol
        className={clsx(
          "pl-5 space-y-2.5 list-decimal text-base tracking-normal dark:text-zinc-400 text-zinc-600 font-normal transition-colors duration-150 ease-in-out my-2",
          "marker:text-zinc-400 dark:marker:text-zinc-600",
          "[blockquote_&]:marker:text-inherit [blockquote_&]:text-inherit",
          "[&>li]:pl-2",
          className
        )}
      >
        {children}
      </ol>
    ),

    ul: ({ children, className }) => (
      <ul
        className={clsx(
          "pl-5 space-y-2.5 lit text-base tracking-normal dark:text-zinc-400 text-zinc-600 font-normal transition-colors duration-150 ease-in-out my-2",
          "marker:text-zinc-400 dark:marker:text-zinc-600",
          "[blockquote_&]:marker:text-inherit [blockquote_&]:text-inherit",
          "[&>li]:pl-2",
          // Level 1
          "list-disc",
          // Level 2
          "[&>li>ul]:list-[circle]",
          // Level 3
          "[&>li>ul>li>ul]:list-[square]",
          // Level 4
          "[&>li>ul>li>ul>li>ul]:list-disc",
          // Level 5
          "[&>li>ul>li>ul>li>ul>li>ul]:list-[circle]",
          // Level 6
          "[&>li>ul>li>ul>li>ul>li>ul>li>ul]:list-[square]",
          className
        )}
      >
        {children}
      </ul>
    ),
  };
}
