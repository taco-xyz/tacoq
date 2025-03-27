// React Imports
import { type PropsWithChildren } from "react";

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
}: PropsWithChildren<{ className?: string }>) {
  return (
    <blockquote
      className={clsx(
        "my-2 flex w-full flex-row items-start gap-x-3 rounded-2xl bg-blue-400/10 px-4.5 py-4 ring-1 ring-blue-400/30 transition-colors duration-150 ease-in-out ring-inset",
        className,
      )}
    >
      <Info className="mt-[2px] size-5 flex-shrink-0 text-blue-400" />
      <div className="flex w-full min-w-0 flex-col gap-y-4 text-base font-normal tracking-normal text-blue-900 transition-colors duration-150 ease-in-out dark:text-blue-200">
        {children}
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
}: PropsWithChildren<{ className?: string }>) {
  return (
    <blockquote
      className={clsx(
        "my-2 flex w-full flex-row items-start gap-x-3 rounded-2xl bg-green-500/10 px-4.5 py-4 ring-1 ring-green-500/20 transition-colors duration-150 ease-in-out ring-inset",
        className,
      )}
    >
      <Lightbulb className="mt-[2px] size-5 flex-shrink-0 text-green-500" />
      <div className="flex w-full min-w-0 flex-col gap-y-4 text-base font-normal tracking-normal text-green-900 transition-colors duration-150 ease-in-out dark:text-green-200">
        {children}
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
}: PropsWithChildren<{ className?: string }>) {
  return (
    <blockquote
      className={clsx(
        "my-2 flex w-full flex-row items-start gap-x-3 rounded-2xl bg-indigo-400/10 px-4.5 py-4 ring-1 ring-indigo-400/30 transition-colors duration-150 ease-in-out ring-inset",
        className,
      )}
    >
      <CircleAlert className="mt-[2px] size-5 flex-shrink-0 text-indigo-400" />
      <div className="flex w-full min-w-0 flex-col gap-y-4 text-base font-normal tracking-normal text-indigo-900 transition-colors duration-150 ease-in-out dark:text-indigo-200">
        {children}
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
}: PropsWithChildren<{ className?: string }>) {
  return (
    <blockquote
      className={clsx(
        "my-2 flex w-full flex-row items-start gap-x-3 rounded-2xl bg-yellow-400/10 px-4.5 py-4 ring-1 ring-yellow-400/25 transition-colors duration-150 ease-in-out ring-inset",
        className,
      )}
    >
      <TriangleAlert className="mt-[2px] size-5 flex-shrink-0 text-yellow-500" />
      <div className="flex w-full min-w-0 flex-col gap-y-4 text-base font-normal tracking-normal text-yellow-900 transition-colors duration-150 ease-in-out dark:text-yellow-100">
        {children}
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
}: PropsWithChildren<{ className?: string }>) {
  return (
    <blockquote
      className={clsx(
        "my-2 flex w-full flex-row items-start gap-x-3 rounded-2xl bg-red-400/10 px-4.5 py-4 ring-1 ring-red-400/25 transition-colors duration-150 ease-in-out ring-inset",
        className,
      )}
    >
      <ShieldAlert className="mt-[2px] size-5 flex-shrink-0 text-red-400" />
      <div className="flex w-full min-w-0 flex-col gap-y-4 text-base font-normal tracking-normal text-red-900 transition-colors duration-150 ease-in-out dark:text-red-200">
        {children}
      </div>
    </blockquote>
  );
}

/**
 * Default blockquote component for standard quotes.
 * Uses neutral gray styling with information icon.
 */
export function Blockquote({
  children,
  className,
}: PropsWithChildren<{ className?: string }>) {
  return (
    <blockquote
      className={clsx(
        "my-2 flex w-full flex-row items-start gap-x-3 rounded-2xl bg-zinc-400/10 px-4.5 py-4 ring-1 ring-zinc-400/30 transition-colors duration-150 ease-in-out ring-inset",
        className,
      )}
    >
      <Info className="mt-[2px] size-5 flex-shrink-0 text-zinc-500 transition-colors duration-150 ease-in-out dark:text-zinc-400" />
      <div className="flex w-full min-w-0 flex-col gap-y-4 text-base font-normal tracking-normal text-zinc-700 transition-colors duration-150 ease-in-out dark:text-zinc-300">
        {children}
      </div>
    </blockquote>
  );
}
