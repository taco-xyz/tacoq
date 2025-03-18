// Next Imports
import Link from "next/link";
import Image from "next/image";

// Types Imports
import type { MDXComponents } from "mdx/types";
import { HeadingTypes, getHeaderId } from "@/types/page/Heading";
import { Card, CardGroup } from "@/components/mdx/Card";

// Components Imports
import Heading from "@/components/mdx/heading/Heading";

// Utils Imports
import clsx from "clsx";

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    ...components,
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

    p: ({ children, className }) => (
      <p
        className={clsx(
          "text-base tracking-normal dark:text-zinc-400 text-zinc-500 font-normal transition-colors duration-150 ease-in-out",
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
          "dark:text-white text-zinc-700 font-semibold border-b dark:border-zinc-300 border-zinc-500 hover:border-b-[2px] transition-colors ease-in-out duration-150",
          className
        )}
      >
        {children}
      </Link>
    ),

    strong: ({ children, className }) => (
      <strong
        className={clsx(
          "font-semibold text-zinc-700 dark:text-white transition-colors duration-150 ease-in-out",
          className
        )}
      >
        {children}
      </strong>
    ),

    em: ({ children, className }) => (
      <em
        className={clsx(
          "italic text-zinc-700 dark:text-white transition-colors duration-150 ease-in-out",
          className
        )}
      >
        {children}
      </em>
    ),

    code: ({ children, className }) => (
      <code
        className={clsx(
          "font-mono text-sm text-zinc-700 dark:text-white font-medium transition-all duration-150 ease-in-out",
          // Code block
          "[pre_&]:py-4 [pre_&]:px-0 [pre_&]:bg-transparent",
          // Inline code
          "py-0.5 px-1.5 rounded-[5px] bg-zinc-100 dark:bg-zinc-800 [&:not(pre_code)]:whitespace-nowrap",
          className
        )}
      >
        {children}
      </code>
    ),

    blockquote: ({ children, className }) => (
      <blockquote
        className={clsx(
          "border-l-4 border-zinc-200 dark:border-zinc-800 pl-4 py-2 transition-colors duration-150 ease-in-out my-1",
          className
        )}
      >
        {children}
      </blockquote>
    ),

    // Cards ---------------------------------------------------

    Card: ({ className, ...props }) => (
      <Card className={className} {...props}></Card>
    ),

    CardGroup: ({ className, ...props }) => (
      <CardGroup className={className} {...props}></CardGroup>
    ),

    hr: ({ className }) => (
      <hr
        className={clsx(
          "border-t border-zinc-200 dark:border-zinc-800 my-6 transition-colors duration-150 ease-in-out",
          className
        )}
      />
    ),

    img: ({ src, alt, className }) => (
      <span className="w-full h-auto aspect-video flex my-2 ring-1 rounded-2xl p-1  ring-zinc-200/70 dark:ring-zinc-800/70 bg-zinc-100 dark:bg-zinc-900 shadow-xl shadow-zinc-100 dark:shadow-black/10 transition-all duration-150 ease-in-out">
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

    ol: ({ children, className }) => (
      <ol
        className={clsx(
          "pl-5 space-y-2.5 list-decimal text-base tracking-normal dark:text-zinc-400 text-zinc-500 font-normal transition-colors duration-150 ease-in-out my-2",
          "marker:text-zinc-700 dark:marker:text-white ",
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
          "pl-5 space-y-2.5 lit text-base tracking-normal dark:text-zinc-400 text-zinc-500 font-normal transition-colors duration-150 ease-in-out my-2",
          "marker:text-zinc-700 dark:marker:text-white",
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

    pre: ({ children, className }) => (
      <pre
        className={clsx(
          className,
          "bg-zinc-50 dark:bg-zinc-900 ring-1 ring-zinc-200 dark:ring-zinc-800 overflow-x-auto rounded-2xl"
        )}
      >
        {children}
      </pre>
    ),
  };
}
