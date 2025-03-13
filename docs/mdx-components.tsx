// Types Imports
import type { MDXComponents } from "mdx/types";
import { HeadingTypes, getHeaderId } from "@/types/page/Heading";

// Components Imports
import Heading from "@/app/components/heading/Heading";

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
      <p className={clsx("text-base tracking-normal dark:text-zinc-400 text-zinc-500 font-normal transition-colors duration-150 ease-in-out", className)}>
        {children}
      </p>
    ),
  };
}
