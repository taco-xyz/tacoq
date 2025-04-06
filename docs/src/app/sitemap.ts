// Next Imports
import { MetadataRoute } from "next";

// Types Imports
import type { PageTree } from "@/types/PageTree";
import type { PageTreeElement } from "@/types/page-tree-element/PageTreeElement";

// Data Imports
import pageTreeJson from "@/page-tree.json";

/**
 * Recursively extracts all URLs from the page tree
 * @param elements - Array of PageTreeElement objects from the page tree
 * @returns Array of URL strings
 */
function getAllUrls(elements: PageTreeElement[]): string[] {
  let urls: string[] = [];

  for (const element of elements) {
    if ("children" in element) {
      urls = urls.concat(getAllUrls(element.children));
    } else {
      urls.push(element.url);
    }
  }

  return urls;
}

/**
 * Enum for valid sitemap change frequencies
 * @see https://www.sitemaps.org/protocol.html#changefreqdef
 */
enum ChangeFrequency {
  ALWAYS = "always",
  HOURLY = "hourly",
  DAILY = "daily",
  WEEKLY = "weekly",
  MONTHLY = "monthly",
  YEARLY = "yearly",
  NEVER = "never",
}

/**
 * Generates the sitemap for the website
 * @returns Sitemap configuration for Next.js
 * @see https://nextjs.org/docs/app/api-reference/file-conventions/metadata/sitemap
 */
export default function sitemap(): MetadataRoute.Sitemap {
  const pageTree = pageTreeJson as PageTree;
  const urls = getAllUrls(pageTree.children);

  return [
    {
      url: "https://www.tacodivision.com",
      lastModified: new Date(),
      changeFrequency: "yearly",
      priority: 1,
    },
    ...urls.map((url) => ({
      url: `https://www.tacodivision.com${url}`,
      lastModified: new Date(),
      changeFrequency: ChangeFrequency.MONTHLY,
      priority: 0.9,
    })),
  ];
}
