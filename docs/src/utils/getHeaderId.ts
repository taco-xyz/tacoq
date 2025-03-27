
// Types Imports
import type { Header } from "@/types/PageTree";

/**
 * Generates a unique ID for a header based on its type and title
 * @param header - The header object containing type and title
 * @returns A unique ID for the header
 */
export function getHeaderId(header: Header) {
  return header.type + "-" + header.title.toLowerCase().replace(/\s+/g, "-");
}
