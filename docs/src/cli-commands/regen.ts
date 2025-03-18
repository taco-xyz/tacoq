// Types ---------------------------------

/**
 * Represents the metadata for a page in JSON format.
 * @property {string} title - The title of the page
 * @property {string} description - A brief description of the page content
 * @property {string} icon - The icon identifier for the page
 */
type MetadataJson = {
  title: string;
  description: string;
  icon: string;
};

/**
 * The type of heading for a content row.
 *
 * Represents HTML heading levels h1-h6.
 *
 * @example
 * ```md
 * # Heading 1     -> HeadingType.H1
 * ## Heading 2    -> HeadingType.H2
 * ### Heading 3   -> HeadingType.H3
 * ```
 */
enum HeadingType {
  H1 = "h1",
  H2 = "h2",
  H3 = "h3",
  H4 = "h4",
  H5 = "h5",
  H6 = "h6",
}

/**
 * Represents a single heading row in a page's content.
 *
 * @property {string} title - The text content of the heading
 * @property {HeadingType} type - The heading level (h1-h6)
 *
 * @example
 * Given MDX:
 * ```mdx
 * ### My Section Title
 * ```
 *
 * Becomes:
 * ```ts
 * {
 *   title: "My Section Title",
 *   type: HeadingType.H3
 * }
 * ```
 */
type ContentRow = {
  title: string;
  type: HeadingType;
};

/**
 * Represents an entry in the documentation structure.
 * An entry can be either a page (with URL) or a folder (with children).
 *
 * @property {string} [url] - The URL path of the page, if it's a page
 * @property {MetadataJson} metadata - The metadata for the entry
 * @property {string} [rawContent] - The raw content of the page, if it's a page
 * @property {ContentRow[]} [contentRows] - The content rows of the page, if it's a page
 * @property {Entry[]} [children] - The child entries, if it's a folder
 */
type Entry = {
  url?: string;
  metadata: MetadataJson;
  rawContent?: string;
  contentRows?: ContentRow[];
  children?: Entry[];
};

/**
 * Represents the entire page tree structure
 */
type PageTree = {
  children: Entry[];
};

// Imports ---------------------------------
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import chalk from "chalk";

// Constants ---------------------------------
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const APP_DIR = path.join(__dirname, "..", "app");

// Helper Functions ---------------------------------

/**
 * Extracts headings from MDX content
 * @param content - The raw MDX content
 * @returns Array of content rows containing headings
 */
function extractHeadings(content: string): ContentRow[] {
  const rows: ContentRow[] = [];
  let inCodeBlock = false;
  // Normalize line endings and split
  const lines = content.replace(/\r\n/g, "\n").split("\n");

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Skip code blocks
    if (line.trim().startsWith("```")) {
      inCodeBlock = !inCodeBlock;
      continue;
    }

    if (inCodeBlock) continue;

    // Match headings at start of line (after any whitespace)
    const match = line.match(/^(\s*)(#{1,6})\s+(.+)$/);
    if (match) {
      const level = match[2].length;
      const title = match[3].trim();
      rows.push({
        title,
        type: `h${level}` as HeadingType,
      });
    }
  }

  return rows;
}

/**
 * Reads metadata from a metadata.json file
 * @param filePath - Path to the metadata.json file
 * @returns The parsed metadata or null if file doesn't exist
 */
function readMetadata(filePath: string): MetadataJson | null {
  try {
    const content = fs.readFileSync(filePath, "utf-8");
    return JSON.parse(content);
  } catch {
    return null;
  }
}

/**
 * Reads content from a page.mdx file
 * @param filePath - Path to the page.mdx file
 * @returns The raw content or null if file doesn't exist
 */
function readPageContent(filePath: string): string | null {
  try {
    return fs.readFileSync(filePath, "utf-8");
  } catch {
    return null;
  }
}

/**
 * Scans a directory for documentation pages
 * @param dirPath - Path to the directory to scan
 * @param isRoot - Whether this is the root app directory
 * @param depth - Current directory depth (for indentation)
 * @returns Array of found entries
 */
function scanDirectory(
  dirPath: string,
  isRoot: boolean = false,
  depth: number = 0
): Entry[] {
  const entries: Entry[] = [];
  const dirEntries = fs.readdirSync(dirPath, { withFileTypes: true });
  const indent = "  ".repeat(depth);

  const metadataPath = path.join(dirPath, "metadata.json");
  const metadata = readMetadata(metadataPath);
  const pagePath = path.join(dirPath, "page.mdx");
  const content = readPageContent(pagePath);

  // Log directory info
  const relativePath = path.relative(APP_DIR, dirPath);
  console.log(chalk.blue(`${indent}📁 Scanning ${relativePath || "root"}`));
  if (metadata) {
    console.log(chalk.gray(`${indent}  Title: ${metadata.title}`));
  } else {
    console.log(chalk.yellow(`${indent}  ⚠️  No metadata.json found`));
  }

  if (content) {
    console.log(chalk.gray(`${indent}  📄 Found page.mdx`));
  } else {
    console.log(chalk.yellow(`${indent}  ⚠️  No page.mdx found`));
  }

  // Scan subdirectories first
  const children: Entry[] = [];
  for (const entry of dirEntries) {
    if (entry.isDirectory()) {
      const fullPath = path.join(dirPath, entry.name);
      children.push(...scanDirectory(fullPath, false, depth + 1));
    }
  }

  // If this is the root directory, just return the children
  if (isRoot) {
    console.log(
      chalk.green(`\n✨ Found ${children.length} top-level sections`)
    );
    return children;
  }

  // If we have children, add them as a folder entry
  if (children.length > 0) {
    entries.push({
      metadata: metadata || {
        title: path.basename(dirPath),
        description: "",
        icon: "",
      },
      children,
    });
  }

  // If we have a page.mdx and metadata, add it as a page entry
  if (content && metadata) {
    entries.push({
      url: `/${relativePath.replace(/\\/g, "/")}`,
      metadata,
      rawContent: content,
      contentRows: extractHeadings(content),
    });
  }

  // Also scan for any .mdx files in the current directory
  const additionalMdxFiles = dirEntries.filter(
    (entry) =>
      !entry.isDirectory() &&
      entry.name.endsWith(".mdx") &&
      entry.name !== "page.mdx"
  );
  if (additionalMdxFiles.length > 0) {
    console.log(
      chalk.gray(
        `${indent}  📄 Found ${additionalMdxFiles.length} additional MDX files`
      )
    );
    for (const entry of additionalMdxFiles) {
      const fullPath = path.join(dirPath, entry.name);
      const relativePath = path.relative(APP_DIR, fullPath);
      const content = readPageContent(fullPath);

      if (content) {
        entries.push({
          url: `/${relativePath.replace(/\\/g, "/").replace(/\.mdx$/, "")}`,
          metadata: {
            title: entry.name.replace(/\.mdx$/, ""),
            description: "",
            icon: "",
          },
          rawContent: content,
          contentRows: extractHeadings(content),
        });
      }
    }
  }

  return entries;
}

/**
 * Prints a summary tree of the page structure
 * @param entries - Array of entries to print
 * @param depth - Current depth for indentation
 */
function printSummaryTree(entries: Entry[], depth: number = 0): void {
  const indent = "  ".repeat(depth);
  for (const entry of entries) {
    const icon = entry.children ? "📚" : "📄";
    const contentRows = entry.contentRows
      ? ` (${chalk.gray(entry.contentRows.length)} headings)`
      : "";
    console.log(
      `${indent}${icon} ${chalk.bold(entry.metadata.title)}${
        entry.url ? ` (${chalk.gray(entry.url)})` : ""
      }${contentRows}`
    );
    if (entry.children) {
      printSummaryTree(entry.children, depth + 1);
    }
  }
}

// Main ---------------------------------

/**
 * Generates the page structure by scanning the app directory
 */
export function generatePageStructure(): PageTree {
  console.log(chalk.cyan("\n🚀 Starting documentation scan..."));
  const entries = scanDirectory(APP_DIR, true);
  console.log(
    chalk.green(
      `\n✅ Scan complete! Found ${entries.length} top-level sections`
    )
  );
  return { children: entries };
}

export function regenPageStructure() {
  console.log(chalk.cyan("\n📝 Regenerating page structure..."));
  const pageTree = generatePageStructure();
  const outputPath = path.join(__dirname, "..", "page-tree.json");
  fs.writeFileSync(outputPath, JSON.stringify(pageTree, null, 2));
  console.log(
    chalk.green(
      `\n💾 Saved page structure to ${path.relative(process.cwd(), outputPath)}`
    )
  );

  console.log(chalk.bold(chalk.magentaBright("\n📊 Generated Structure:\n")));
  printSummaryTree(pageTree.children);
  console.log("\n");
}
