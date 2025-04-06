/**
 * Metadata describing a page's properties
 */
export type Metadata = {
  /** Title text of the page */
  title: string;
  /** Index value for sorting pages */
  index: number;
  /** Optional description text of the page */
  description?: string;
  /** Optional icon component to show in the page */
  icon?: string;
};
