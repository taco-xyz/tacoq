// Types Imports
import { Metadata } from "@/types/page-tree-element/Metadata";
import { PageTreeElement } from "@/types/page-tree-element/PageTreeElement";

/**
 * Represents a folder in the navigation tree
 */
export type Folder = {
  /** Metadata describing the folder */
  metadata: Metadata;
  /** Child pages */
  children: PageTreeElement[];
};
