// Next Imports
import { MetadataRoute } from "next";

// Types Imports
import type { Page, PageTree } from "@/types/PageTree";

// Data Imports
import pageTreeJson from "@/page-tree.json";

/**
 * Recursively extracts all URLs from the page tree
 * @param pages - Array of Page objects from the page tree
 * @returns Array of URL strings
 */
function getAllUrls(pages: Page[]): string[] {
  let urls: string[] = [];

  for (const page of pages) {
    if (page.url) {
      urls.push(page.url);
    }
    if (page.children) {
      urls = urls.concat(getAllUrls(page.children));
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
