// Types Imports
import { Header } from "@/types/page-tree-element/Header";
import { Metadata } from "@/types/page-tree-element/Metadata";

/**
 * Represents a page in the navigation tree
 */
export type Page = {
  /** URL path for the page */
  url: string;
  /** Metadata describing the page */
  metadata: Metadata;
  /** Raw content for the page */
  rawContent: string;
  /** Headers for the page */
  headers: Header[];
  /** Last updated date for the page */
  lastUpdated: string;
};
