/**
 * Represents a navigation anchor link
 */
export interface Anchor {
  /** Text to display for the anchor */
  title: string;
  /** URL the anchor links to */
  url: string;
  /** Optional icon component */
  Icon?: React.ComponentType<{ className?: string }>;
}
