import type { Heading } from "./Heading";

/**
 * Represents a page in the navigation tree
 */
export interface Page {
  /** Optional URL path for the page */
  url?: string;
  /** Metadata describing the page */
  metadata: PageMetadata;
  /** Optional array of child pages */
  children?: Page[];
  /** Optional content for the page */
  content?: Heading[];
}

/**
 * Metadata describing a page's properties
 */
export interface PageMetadata {
  /** Title text of the page */
  title: string;
  /** Optional description text */
  description?: string;
  /** Optional badge to display within the page */
  badge?: PageBadgeMetadata;
  /** Optional sidebar configuration */
  sidebar?: PageSidebarMetadata;
}

/**
 * Configuration for a page's badge
 */
export interface PageBadgeMetadata {
  /** Text to show in the badge */
  text: string;
  /** Optional icon component to show in the badge */
  Icon?: React.ComponentType<{ className?: string }>;
}

/**
 * Configuration for a page's sidebar display
 */
export interface PageSidebarMetadata {
  /** Optional title text for the sidebar */
  title?: string;
  /** Optional icon component to show in the sidebar */
  Icon?: React.ComponentType<{ className?: string }>;
}
