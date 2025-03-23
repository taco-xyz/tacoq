// React Imports
import { type ReactNode } from "react";

// Lucide Icons
import {
  Info,
  TriangleAlert,
  CircleAlert,
  Lightbulb,
  ShieldAlert,
} from "lucide-react";

// Utility Imports
import clsx from "clsx";

/**
 * A collection of callout components for highlighting different types of content.
 * Each component follows GitHub-style callout patterns with consistent styling and icons.
 *
 * @example Basic usage
 * ```tsx
 * <Note>This is a note</Note>
 * <Warning>This is a warning</Warning>
 * ```
 *
 * @example With markdown content
 * ```tsx
 * <Tip>
 *   Helpful advice with **bold** and _italic_ text
 *   - List items work too
 *   - Another item
 * </Tip>
 * ```
 */

/**
 * Note callout for general information and neutral content.
 * Uses blue color scheme with information icon.
 */
export function Note({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <blockquote
      className={clsx(
        "ring-1 px-4.5 py-4 my-2 rounded-2xl ring-inset ring-blue-400/30 bg-blue-400/10",
        className
      )}
    >
      <div className="flex flex-row items-start gap-x-3">
        <Info className="size-5 text-blue-400 mt-[2px] flex-shrink-0" />
        <div className="text-base tracking-normal flex flex-col gap-y-4 font-normal dark:text-blue-200 text-blue-900">
          {children}
        </div>
      </div>
    </blockquote>
  );
}

/**
 * Tip callout for helpful advice and suggestions.
 * Uses green color scheme with lightbulb icon.
 */
export function Tip({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <blockquote
      className={clsx(
        "ring-1 px-4.5 py-4 my-2 rounded-2xl ring-inset ring-green-500/20 bg-green-500/10",
        className
      )}
    >
      <div className="flex flex-row items-start gap-x-3">
        <Lightbulb className="size-5 text-green-500 mt-[2px] flex-shrink-0" />
        <div className="text-base tracking-normal flex flex-col gap-y-4 font-normal dark:text-green-200 text-green-900">
          {children}
        </div>
      </div>
    </blockquote>
  );
}

/**
 * Important callout for key information that shouldn't be missed.
 * Uses indigo color scheme with exclamation circle icon.
 */
export function Important({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <blockquote
      className={clsx(
        "ring-1 px-4.5 py-4 my-2 rounded-2xl ring-inset ring-indigo-400/30 bg-indigo-400/10",
        className
      )}
    >
      <div className="flex flex-row items-start gap-x-3">
        <CircleAlert className="size-5 text-indigo-400 mt-[2px] flex-shrink-0" />
        <div className="text-base tracking-normal flex flex-col gap-y-4 font-normal dark:text-indigo-200 text-indigo-900">
          {children}
        </div>
      </div>
    </blockquote>
  );
}

/**
 * Warning callout for potential issues that need attention.
 * Uses yellow color scheme with exclamation triangle icon.
 */
export function Warning({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <blockquote
      className={clsx(
        "ring-1 px-4.5 py-4 my-2 rounded-2xl ring-inset ring-yellow-400/25 bg-yellow-400/10",
        className
      )}
    >
      <div className="flex flex-row items-start gap-x-3">
        <TriangleAlert className="size-5 text-yellow-500 mt-[2px] flex-shrink-0" />
        <div className="text-base tracking-normal flex flex-col gap-y-4 font-normal dark:text-yellow-100 text-yellow-900">
          {children}
        </div>
      </div>
    </blockquote>
  );
}

/**
 * Caution callout for dangerous operations or critical warnings.
 * Uses red color scheme with shield exclamation icon.
 */
export function Caution({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <blockquote
      className={clsx(
        "ring-1 px-4.5 py-4 my-2 rounded-2xl ring-inset ring-red-400/25 bg-red-400/10",
        className
      )}
    >
      <div className="flex flex-row items-start gap-x-3">
        <ShieldAlert className="size-5 text-red-400 mt-[2px] flex-shrink-0" />
        <div className="text-base tracking-normal flex flex-col gap-y-4 font-normal dark:text-red-200 text-red-900">
          {children}
        </div>
      </div>
    </blockquote>
  );
}

/**
 * Default blockquote component for standard quotes.
 * Uses neutral gray styling with information icon.
 */
export default function Blockquote({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <blockquote
      className={clsx(
        "ring-1 px-4.5 py-4 transition-colors duration-150 ease-in-out my-2 rounded-2xl bg-zinc-400/10 ring-inset ring-zinc-400/30",
        className
      )}
    >
      <div className="flex flex-row items-start gap-x-3">
        <Info className="size-5 mt-[2px] flex-shrink-0 text-zinc-500 dark:text-zinc-400 transition-colors duration-150 ease-in-out" />
        <div className="text-zinc-700 dark:text-zinc-300 flex flex-col gap-y-4">
          {children}
        </div>
      </div>
    </blockquote>
  );
}
