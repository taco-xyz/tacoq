// Next Imports
import { MetadataRoute } from "next";

/**
 * Generates the robots.txt file for the website
 * @returns Robots.txt configuration for Next.js
 * @see https://nextjs.org/docs/app/api-reference/file-conventions/metadata/robots
 */
export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: "*",
      allow: "/",
      disallow: [], // No disallowed paths
    },
    sitemap: "https://www.tacodivision.com/sitemap.xml", // Add your sitemap URL
    host: "https://www.tacodivision.com", // Add your domain
  };
}
