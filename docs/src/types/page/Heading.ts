/**
 * Enum representing different heading levels in the document
 */
export enum HeadingTypes {
  H1 = "h1",
  H2 = "h2",
  H3 = "h3",
  H4 = "h4",
  H5 = "h5",
  H6 = "h6",
}

/**
 * Interface for document headings
 */
export interface Heading {
  /** The heading level (h1, h2, h3) */
  type: HeadingTypes;
  /** The text content of the heading */
  name: string;
}

export function getHeaderId(heading: Heading) {
  return heading.type + "-" + heading.name.toLowerCase().replace(/\s+/g, "-");
}
