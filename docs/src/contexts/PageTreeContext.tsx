"use client";

// React Imports
import {
  createContext,
  useContext,
  useState,
  useCallback,
  useMemo,
} from "react";

// Next Imports
import { usePathname } from "next/navigation";

// Type imports
import type { Page } from "@/types/page/Page";
import type { Anchor } from "@/types/Anchor";
import { HeadingTypes } from "@/types/page/Heading";
import FooterContent from "@/types/FooterContent";

// Heroicons Imports
import {
  HomeIcon,
  NewspaperIcon,
  CreditCardIcon,
} from "@heroicons/react/24/solid";
import {
  ArrowUpTrayIcon,
  BoltIcon,
  CodeBracketIcon,
  Cog6ToothIcon,
  ComputerDesktopIcon,
  LinkIcon,
  PencilIcon,
  RocketLaunchIcon,
} from "@heroicons/react/24/outline";

// Custom Icons Imports
import { GithubIcon, XIcon, DiscordIcon } from "@/app/components/icons/social";

const anchors: Anchor[] = [
  {
    title: "Home",
    url: "/",
    Icon: HomeIcon,
  },
  {
    title: "Blog",
    url: "/blog",
    Icon: NewspaperIcon,
  },
  {
    title: "Pricing",
    url: "/pricing",
    Icon: CreditCardIcon,
  },
];

const pages: Page[] = [
  {
    metadata: {
      title: "Getting Started",
      description: "Start building your app with the following steps.",
      sidebar: {
        title: "Getting Started",
        Icon: BoltIcon,
      },
    },
    children: [
      {
        url: "/getting-started/quickstart",
        metadata: {
          title: "Quickstart",
          description:
            "Start building your app with the following steps. Follow these tutorials to get started and you'll be up and running in no time.",
          badge: {
            text: "Getting Started",
          },

          sidebar: {
            title: "Quickstart",
            Icon: RocketLaunchIcon,
          },
        },
        content: [
          { type: HeadingTypes.H1, name: "Quickstart Guide" },
          { type: HeadingTypes.H2, name: "Installation" },
          { type: HeadingTypes.H2, name: "Basic Setup" },
          { type: HeadingTypes.H3, name: "Configuration" },
          { type: HeadingTypes.H3, name: "Environment Variables" },
          { type: HeadingTypes.H2, name: "Next Steps" },
        ],
      },
      {
        url: "/getting-started/editing",
        metadata: {
          title: "Editing",
          description: "Edit your content with the web app or local editor.",
          badge: {
            text: "Getting Started",
          },
          sidebar: {
            title: "Editing",
            Icon: PencilIcon,
          },
        },
        content: [
          { type: HeadingTypes.H1, name: "Editing Your Documentation" },
          { type: HeadingTypes.H2, name: "Available Editors" },
          { type: HeadingTypes.H2, name: "File Structure" },
          { type: HeadingTypes.H2, name: "Best Practices" },
        ],
        children: [
          {
            url: "/getting-started/editing/local",
            metadata: {
              title: "Local Development",
              description: "Edit your content with the local editor.",
              badge: {
                text: "Getting Started",
              },
              sidebar: {
                title: "Local Development",
                Icon: CodeBracketIcon,
              },
            },
            content: [
              { type: HeadingTypes.H1, name: "Local Development Setup" },
              { type: HeadingTypes.H2, name: "Prerequisites" },
              { type: HeadingTypes.H2, name: "Editor Configuration" },
              { type: HeadingTypes.H3, name: "VS Code Setup" },
              { type: HeadingTypes.H3, name: "Extensions" },
              { type: HeadingTypes.H2, name: "Development Workflow" },
            ],
          },
          {
            url: "/getting-started/editing/web",
            metadata: {
              title: "Web Editor",
              description: "Edit your content with the web app.",
              badge: {
                text: "Getting Started",
              },
              sidebar: {
                title: "Web Editor",
                Icon: ComputerDesktopIcon,
              },
            },
            content: [
              { type: HeadingTypes.H1, name: "Web Editor Guide" },
              { type: HeadingTypes.H2, name: "Getting Started" },
              { type: HeadingTypes.H2, name: "Interface Overview" },
              { type: HeadingTypes.H3, name: "Toolbar" },
              { type: HeadingTypes.H3, name: "Preview Panel" },
              { type: HeadingTypes.H2, name: "Collaboration Features" },
            ],
          },
        ],
      },
      {
        url: "/getting-started/settings",
        metadata: {
          title: "Global Settings",
          description: "Edit your global settings.",
          badge: {
            text: "Getting Started",
          },
          sidebar: {
            title: "Global Settings",
            Icon: Cog6ToothIcon,
          },
        },
      },
      {
        url: "/getting-started/navigation",
        metadata: {
          title: "Navigation",
          description: "You can customize the navigation of your docs.",
          badge: {
            text: "Getting Started",
          },
          sidebar: {
            title: "Navigation",
            Icon: LinkIcon,
          },
        },
      },
      {
        url: "/getting-started/migration",
        metadata: {
          title: "Migration",
          description: "Migrate your docs to Mintlify.",
          badge: {
            text: "Getting Started",
          },
          sidebar: {
            title: "Migration",
            Icon: ArrowUpTrayIcon,
          },
        },
      },
    ],
  },
  {
    metadata: {
      title: "Writing Content",
      badge: {
        text: "Writing Content",
        Icon: BoltIcon,
      },
      description: "Start writing your content with the following steps.",
    },
    children: [
      {
        url: "/writing/page-titles",
        metadata: {
          title: "Page Titles",
        },
      },
      {
        url: "/writing/metadata",
        metadata: {
          title: "Metadata",
        },
      },
      {
        url: "/writing/headers-and-text",
        metadata: {
          title: "Headers and Text",
        },
      },
      {
        url: "/writing/tables",
        metadata: {
          title: "Tables",
        },
      },
    ],
  },
];

const footerContent: FooterContent = {
  linkGroups: [
    {
      groupName: "Product",
      links: [
        { linkName: "Features", url: "/features" },
        { linkName: "Pricing", url: "/pricing" },
        { linkName: "Documentation", url: "/docs" },
      ],
    },
    {
      groupName: "Resources",
      links: [
        { linkName: "Blog", url: "/blog" },
        { linkName: "Support", url: "/support" },
        { linkName: "API", url: "/api" },
      ],
    },
    {
      groupName: "Company",
      links: [
        { linkName: "About", url: "/about" },
        { linkName: "Careers", url: "/careers" },
        { linkName: "Contact", url: "/contact" },
      ],
    },
    {
      groupName: "Legal",
      links: [
        { linkName: "Privacy", url: "/privacy" },
        { linkName: "Terms", url: "/terms" },
        { linkName: "Security", url: "/security" },
      ],
    },
  ],
  socialLinks: [
    { Icon: GithubIcon, url: "https://github.com/your-repo" },
    { Icon: XIcon, url: "https://twitter.com/your-handle" },
    { Icon: DiscordIcon, url: "https://discord.gg/your-server" },
  ],
};

/**
 * Context for managing the page navigation tree state
 */
interface PageTreeContextType {
  /** Expands a page in the navigation tree */
  expandPage: (pageTitle: string) => void;
  /** Collapses a page in the navigation tree */
  collapsePage: (pageTitle: string) => void;
  /** Checks if a page is currently expanded */
  isPageExpanded: (pageTitle: string) => boolean;
  /** Retrieves a page by its title */
  getPageByTitle: (pageTitle: string) => Page | null;
  /** Title of the currently active page */
  currentPageTitle: string | null;
  /** Array of page titles that are currently visible */
  visiblePagesTitles: string[];
  /** Nested array of all pages in the navigation tree */
  pages: Page[];
  /** Array of anchor links */
  anchors: Anchor[];
  /** Footer content */
  footerContent: FooterContent;
}

const PageTreeContext = createContext<PageTreeContextType | null>(null);

// Helper functions
/**
 * Gets an array of page titles that are currently visible in the navigation tree
 * @param pages - Array of pages to traverse
 * @param expandedPages - Set of page titles that are currently expanded
 * @returns Array of visible page titles in display order
 */
function getVisiblePages(pages: Page[], expandedPages: Set<string>): string[] {
  const visiblePages: string[] = [];
  function traverse(page: Page) {
    visiblePages.push(page.metadata.title);
    if (page.children && expandedPages.has(page.metadata.title)) {
      page.children.forEach(traverse);
    }
  }
  pages.forEach(traverse);
  return visiblePages;
}

/**
 * Finds a page in the navigation tree by its title
 * @param pages - Array of pages to search
 * @param title - Title of the page to find
 * @returns The matching page or null if not found
 */
function findPageByTitle(pages: Page[], title: string): Page | null {
  for (const page of pages) {
    if (page.metadata.title === title) return page;
    if (page.children) {
      const found = findPageByTitle(page.children, title);
      if (found) return found;
    }
  }
  return null;
}

/**
 * Finds a page by URL and returns array of parent page titles leading to it
 * @param pages - Array of pages to search
 * @param targetUrl - URL of the page to find
 * @param parents - Accumulator for parent page titles (used in recursion)
 * @returns Array of page titles from root to target page, empty if not found
 */
function findPageAndParents(
  pages: Page[],
  targetUrl: string,
  parents: string[] = []
): string[] {
  for (const page of pages) {
    if (page.url === targetUrl) {
      return [...parents, page.metadata.title];
    }
    if (page.children) {
      const found = findPageAndParents(page.children, targetUrl, [
        ...parents,
        page.metadata.title,
      ]);
      if (found.length > 0) return found;
    }
  }
  return [];
}

export function PageTreeProvider({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();

  // A set of page titles that are currently expanded
  const [expandedPages, setExpandedPages] = useState<Set<string>>(() => {
    const parentPages = findPageAndParents(pages, pathname);
    return new Set(parentPages.slice(0, -1));
  });

  /**
   * Memoized title of the current page based on URL pathname
   * Returns null if no matching page is found
   */
  const currentPageTitle = useMemo(() => {
    function findPageByUrl(pages: Page[], url: string): Page | null {
      for (const page of pages) {
        if (page.url === url) return page;
        if (page.children) {
          const found = findPageByUrl(page.children, url);
          if (found) return found;
        }
      }
      return null;
    }
    const currentPage = findPageByUrl(pages, pathname);
    return currentPage?.metadata.title ?? null;
  }, [pathname]);

  /**
   * Memoized array of page titles that are visible in the navigation
   */
  const visiblePagesTitles = useMemo(
    () => getVisiblePages(pages, expandedPages),
    [expandedPages]
  );

  /**
   * Expands a page in the navigation tree given its title
   */
  const expandPage = useCallback((pageTitle: string) => {
    setExpandedPages((prev) => new Set([...prev, pageTitle]));
  }, []);

  /**
   * Collapses a page in the navigation tree given its title
   */
  const collapsePage = useCallback((pageTitle: string) => {
    setExpandedPages((prev) => {
      const next = new Set(prev);
      next.delete(pageTitle);
      return next;
    });
  }, []);

  /**
   * Checks if a page is currently expanded in the navigation tree
   */
  const isPageExpanded = useCallback(
    (pageTitle: string) => expandedPages.has(pageTitle),
    [expandedPages]
  );

  /**
   * Finds and returns a page by its title
   * Returns null if no matching page is found
   */
  const getPageByTitle = useCallback(
    (pageTitle: string) => findPageByTitle(pages, pageTitle),
    []
  );

  return (
    <PageTreeContext.Provider
      value={{
        expandPage,
        collapsePage,
        isPageExpanded,
        getPageByTitle,
        currentPageTitle,
        visiblePagesTitles,
        pages,
        anchors,
        footerContent,
      }}
    >
      {children}
    </PageTreeContext.Provider>
  );
}

export function usePageTree() {
  const context = useContext(PageTreeContext);
  if (!context) {
    throw new Error("usePageTree must be used within PageTreeProvider");
  }
  return context;
}
