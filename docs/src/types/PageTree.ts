// Types Imports
import type { PageTreeElement } from "@/types/page-tree-element/PageTreeElement";

/**
 * Represents the tree structure of pages
 */
export type PageTree = {
  /** Array of child pages */
  children: PageTreeElement[];
};
