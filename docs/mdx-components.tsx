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
          "my-6 border-t border-zinc-200 transition-colors duration-150 ease-in-out dark:border-zinc-800",
          className,
        )}
      />
    ),

    // IMAGE ---------------------------------------------------------------
    img: ({ src, alt, className }) => (
      <span className="my-2 flex aspect-video h-auto w-full rounded-2xl p-1.5 shadow-xl ring-1 shadow-zinc-700/1 ring-zinc-200 transition-all duration-150 ease-in-out ring-inset dark:shadow-black/5 dark:ring-zinc-800/70">
        <span className="relative h-full w-full overflow-hidden rounded-[11px] border-1 border-zinc-200 transition-all duration-150 ease-in-out dark:border-zinc-800/70">
          <Image
            src={src}
            alt={alt}
            fill
            className={clsx("object-cover object-center", className)}
          />
        </span>
      </span>
    ),

    // CODEBLOCK WRAPPER --------------------------------------------------
    pre: ({ children, className }) => (
      <div className="flex h-fit w-full items-center justify-center rounded-2xl p-1.5 shadow-xl ring-1 shadow-zinc-700/3 ring-zinc-200 transition-all duration-150 ease-in-out ring-inset dark:shadow-black/5 dark:ring-zinc-800/70">
        <pre
          className={clsx(
            className,
            "w-full overflow-x-auto rounded-[11px] bg-zinc-700 shadow-2xl ring-1 shadow-zinc-700/3 ring-zinc-300 transition-all duration-150 ease-in-out dark:bg-zinc-900 dark:shadow-black/5 dark:ring-zinc-800",
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
          "font-mono text-sm font-medium text-[#ce9178] transition-all duration-150 ease-in-out",
          // Code block
          "[pre_&]:bg-transparent [pre_&]:px-0 [pre_&]:py-4",
          // Inline code
          "rounded-[5px] bg-zinc-100 px-1.5 py-0.5 dark:bg-zinc-800 [&:not(pre_code)]:mx-1 [&:not(pre_code)]:break-words",
          // If the code is inside a blockquote, it should inherit the blockquote styles
          "[blockquote_&]:bg-inherit [blockquote_&]:text-inherit [blockquote_&]:ring-1 [blockquote_&]:ring-inherit [blockquote_&]:ring-inset",
          className,
        )}
      >
        {children}
      </code>
    ),

    p: ({ children, className }) => (
      <p
        className={clsx(
          "text-base font-normal tracking-normal text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-400",
          "[blockquote_&]:text-inherit",
          className,
        )}
      >
        {children}
      </p>
    ),

    a: ({ children, className, href }) => (
      <Link
        href={href}
        className={clsx(
          "custom-tab-outline-offset-2 border-b border-blue-500 font-semibold text-zinc-800 transition-[outline] duration-150 ease-in-out hover:border-b-[2px] focus-visible:rounded-sm dark:border-blue-400 dark:text-white",
          "[blockquote_&]:border-inherit [blockquote_&]:text-inherit",
          className,
        )}
      >
        {children}
      </Link>
    ),

    strong: ({ children, className }) => (
      <strong
        className={clsx(
          "font-semibold text-zinc-800 transition-colors duration-150 ease-in-out dark:text-white",
          "[blockquote_&]:text-inherit",
          className,
        )}
      >
        {children}
      </strong>
    ),

    em: ({ children, className }) => (
      <em
        className={clsx(
          "text-zinc-800 italic transition-colors duration-150 ease-in-out dark:text-white",
          "[blockquote_&]:text-inherit",
          className,
        )}
      >
        {children}
      </em>
    ),

    ol: ({ children, className }) => (
      <ol
        className={clsx(
          "my-2 list-decimal space-y-2.5 pl-5 text-base font-normal tracking-normal text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-400",
          "marker:text-zinc-400 dark:marker:text-zinc-600",
          "[blockquote_&]:text-inherit [blockquote_&]:marker:text-inherit",
          "[&>li]:pl-2",
          className,
        )}
      >
        {children}
      </ol>
    ),

    ul: ({ children, className }) => (
      <ul
        className={clsx(
          "lit my-2 space-y-2.5 pl-5 text-base font-normal tracking-normal text-zinc-600 transition-colors duration-150 ease-in-out dark:text-zinc-400",
          "marker:text-zinc-400 dark:marker:text-zinc-600",
          "[blockquote_&]:text-inherit [blockquote_&]:marker:text-inherit",
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
          className,
        )}
      >
        {children}
      </ul>
    ),
  };
}
