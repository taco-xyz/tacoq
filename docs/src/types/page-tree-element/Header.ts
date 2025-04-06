/**
 * Type representing different header levels in the document
 */
export type HeaderType = "h1" | "h2" | "h3" | "h4" | "h5" | "h6";

/**
 * Interface for document headers
 */
export type Header = {
  /** Title text of the header */
  title: string;
  /** The header level (h1, h2, h3) */
  type: HeaderType;
};
