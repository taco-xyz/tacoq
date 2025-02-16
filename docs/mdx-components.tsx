// Types Imports
import type { MDXComponents } from "mdx/types";
import { HeadingTypes, getHeaderId } from "@/types/page/Heading";

// Components Imports
import Heading from "@/app/components/heading/Heading";

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    ...components,
    h1: ({ children }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H1, name: children })}
        Level={HeadingTypes.H1}
      >
        {children}
      </Heading>
    ),

    h2: ({ children }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H2, name: children })}
        Level={HeadingTypes.H2}
      >
        {children}
      </Heading>
    ),

    h3: ({ children }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H3, name: children })}
        Level={HeadingTypes.H3}
      >
        {children}
      </Heading>
    ),

    h4: ({ children }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H4, name: children })}
        Level={HeadingTypes.H4}
      >
        {children}
      </Heading>
    ),

    h5: ({ children }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H5, name: children })}
        Level={HeadingTypes.H5}
      >
        {children}
      </Heading>
    ),

    h6: ({ children }) => (
      <Heading
        id={getHeaderId({ type: HeadingTypes.H6, name: children })}
        Level={HeadingTypes.H6}
      >
        {children}
      </Heading>
    ),

    p: ({ children }) => (
      <p className="text-base tracking-normal dark:text-zinc-400 text-zinc-500 font-normal transition-colors duration-150 ease-in-out">
        {children}
      </p>
    ),
  };
}
