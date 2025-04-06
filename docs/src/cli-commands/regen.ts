// Types ---------------------------------
import type { PageTree } from "@/types/PageTree";
import type { PageTreeElement } from "@/types/page-tree-element/PageTreeElement";
import type { Metadata } from "@/types/page-tree-element/Metadata";
import type { Header, HeaderType } from "@/types/page-tree-element/Header";

// Imports ---------------------------------
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import chalk from "chalk";
import { Page } from "@/types/page-tree-element/Page";
import { Folder } from "@/types/page-tree-element/Folder";

// Constants ---------------------------------
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const APP_DIR = path.join(__dirname, "..", "app");

// Helper Functions ---------------------------------

/**
 * Extracts headers from MDX content
 * @param content - The raw MDX content
 * @returns Array of content rows containing headers
 */
function extractHeaders(content: string): Header[] {
  const rows: Header[] = [];
  const lines = content.replace(/\r\n/g, "\n").split("\n");
  let inCodeBlock = false;

  for (const line of lines) {
    if (line.startsWith("```")) {
      inCodeBlock = !inCodeBlock;
      continue;
    }

    if (inCodeBlock) continue;

    const match = line.match(/^(\s*)(#{1,6})\s+(.+)$/);
    if (!match) continue;

    const [, , level, title] = match;
    rows.push({
      title,
      type: ("h" + level.length) as HeaderType,
    });
  }

  return rows;
}

/**
 * Reads metadata from a metadata.json file
 * @param filePath - Path to the metadata.json file
 * @returns The parsed metadata or null if file doesn't exist
 */
function readMetadata(filePath: string): Metadata | null {
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
  depth: number = 0,
): PageTreeElement[] {
  const entries: PageTreeElement[] = [];
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
  const children: PageTreeElement[] = [];
  for (const entry of dirEntries) {
    if (!entry.isDirectory()) continue;

    const fullPath = path.join(dirPath, entry.name);
    children.push(...scanDirectory(fullPath, false, depth + 1));
  }

  // If this is the root directory, just return the children sorted by index
  if (isRoot) {
    console.log(
      chalk.green(`\n✨ Found ${children.length} top-level sections`),
    );
    return children.sort((a, b) => a.metadata.index - b.metadata.index);
  }

  // If we have children, add them as a folder entry
  if (children.length > 0) {
    const element: Folder = {
      metadata: metadata || {
        title: path.basename(dirPath),
        index: 0,
      },
      children: children.sort((a, b) => a.metadata.index - b.metadata.index),
    };
    entries.push(element);
  } else if (content && metadata) {
    const element: Page = {
      url: `/${relativePath.replace(/\\/g, "/")}`,
      metadata,
      rawContent: content,
      headers: extractHeaders(content),
      lastUpdated: new Date().toISOString(),
    };
    entries.push(element);
  }

  // Sort entries by index before returning
  return entries.sort((a, b) => a.metadata.index - b.metadata.index);
}

/**
 * Prints a summary tree of the page structure
 * @param entries - Array of entries to print
 * @param depth - Current depth for indentation
 */
function printSummaryTree(entries: PageTreeElement[], depth: number = 0): void {
  const indent = "  ".repeat(depth);
  for (const entry of entries) {
    const isFolder = "children" in entry;
    const icon = isFolder ? "📚" : "📄";
    const headers = isFolder
      ? ` (${chalk.gray(entry.children.length)} children)`
      : "";
    console.log(
      `${indent}${icon} ${chalk.bold(entry.metadata.title)}${
        !isFolder ? ` (${chalk.gray(entry.url)})` : ""
      }${headers}`,
    );
    if (isFolder) {
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
      `\n✅ Scan complete! Found ${entries.length} top-level sections`,
    ),
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
      `\n💾 Saved page structure to ${path.relative(process.cwd(), outputPath)}`,
    ),
  );

  console.log(chalk.bold(chalk.magentaBright("\n📊 Generated Structure:\n")));
  printSummaryTree(pageTree.children);
  console.log("\n");
}
