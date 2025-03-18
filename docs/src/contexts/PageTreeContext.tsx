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
import type { Page, PageTree } from "@/types/PageTree";
import type { Anchor } from "@/types/Anchor";
import FooterContent from "@/types/FooterContent";

// Heroicons Imports
import {
  HomeIcon,
  NewspaperIcon,
  CreditCardIcon,
} from "@heroicons/react/24/solid";

// Custom Icons Imports
import {
  GithubIcon,
  XIcon,
  DiscordIcon,
} from "@/components/react/icons/social";

// Data imports
import pageTreeJson from "@/page-tree.json";

const pageTree = pageTreeJson as PageTree;

// Utils
import { getIcon } from "../utils/getIcon";

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
  /** Page one level upwards in the navigation tree */
  parentPageTitle: string | null;
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
  /** Array of pages that are the breadcrumbs for the current page */
  breadcrumbs: Page[];
  /** Previous page in navigation sequence, null if none */
  previousPage: Page | null;
  /** Next page in navigation sequence, null if none */
  nextPage: Page | null;
}

const PageTreeContext = createContext<PageTreeContextType | null>(null);

// Helper functions
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

function getFlattenedPages(pages: Page[]): Page[] {
  const flattened: Page[] = [];
  function traverse(page: Page) {
    if (page.url) {
      flattened.push(page);
    }
    if (page.children) {
      page.children.forEach(traverse);
    }
  }
  pages.forEach(traverse);
  return flattened;
}

export function PageTreeProvider({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();

  const [expandedPages, setExpandedPages] = useState<Set<string>>(() => {
    const parentPages = findPageAndParents(pageTree.children, pathname);
    return new Set(parentPages.slice(0, -1));
  });

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
    const currentPage = findPageByUrl(pageTree.children, pathname);
    return currentPage?.metadata.title ?? null;
  }, [pathname]);

  const visiblePagesTitles = useMemo(
    () => getVisiblePages(pageTree.children, expandedPages),
    [expandedPages]
  );

  const expandPage = useCallback((pageTitle: string) => {
    setExpandedPages((prev) => new Set([...prev, pageTitle]));
  }, []);

  const collapsePage = useCallback((pageTitle: string) => {
    setExpandedPages((prev) => {
      const next = new Set(prev);
      next.delete(pageTitle);
      return next;
    });
  }, []);

  const isPageExpanded = useCallback(
    (pageTitle: string) => expandedPages.has(pageTitle),
    [expandedPages]
  );

  const getPageByTitle = useCallback(
    (pageTitle: string) => findPageByTitle(pageTree.children, pageTitle),
    []
  );

  const breadcrumbs = useMemo(() => {
    const parentTitles = findPageAndParents(pageTree.children, pathname);
    return parentTitles
      .map((title) => findPageByTitle(pageTree.children, title))
      .filter((page): page is Page => !!page);
  }, [pathname]);

  const { previousPage, nextPage } = useMemo(() => {
    const flattenedPages = getFlattenedPages(pageTree.children);
    const currentIndex = flattenedPages.findIndex(
      (page) => page.url === pathname
    );

    return {
      previousPage: currentIndex > 0 ? flattenedPages[currentIndex - 1] : null,
      nextPage:
        currentIndex < flattenedPages.length - 1
          ? flattenedPages[currentIndex + 1]
          : null,
    };
  }, [pathname]);

  const parentPageTitle = useMemo(() => {
    const parentTitles = findPageAndParents(pageTree.children, pathname);
    return parentTitles.length > 1
      ? parentTitles[parentTitles.length - 2]
      : null;
  }, [pathname]);

  // Convert icon names to components
  const pagesWithIcons = useMemo(() => {
    function addIconsToPages(pages: Page[]): Page[] {
      return pages.map((page) => ({
        ...page,
        metadata: {
          ...page.metadata,
          sidebar: {
            title: page.metadata.title,
            Icon: getIcon(page.metadata.icon),
          },
        },
        children: page.children ? addIconsToPages(page.children) : undefined,
      }));
    }
    return addIconsToPages(pageTree.children);
  }, []);

  return (
    <PageTreeContext.Provider
      value={{
        expandPage,
        collapsePage,
        isPageExpanded,
        getPageByTitle,
        parentPageTitle,
        currentPageTitle,
        visiblePagesTitles,
        pages: pagesWithIcons,
        anchors,
        footerContent,
        breadcrumbs,
        previousPage,
        nextPage,
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
