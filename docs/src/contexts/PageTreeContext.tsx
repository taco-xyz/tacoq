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
  BoltIcon,
  LinkIcon,
  RocketLaunchIcon,
  AcademicCapIcon,
  ArrowDownTrayIcon,
  BookOpenIcon,
  BuildingLibraryIcon,
  CpuChipIcon,
  AdjustmentsVerticalIcon,
  ArrowUpIcon,
  ChevronDoubleRightIcon,
  PencilSquareIcon,
  PresentationChartLineIcon,
  ChartBarIcon,
} from "@heroicons/react/24/outline";

// Custom Icons Imports
import {
  GithubIcon,
  XIcon,
  DiscordIcon,
} from "@/components/react/icons/social";

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
  // Quickstart
  {
    metadata: {
      title: "Quickstart",
      description: "Get started using TacoQ",
      sidebar: {
        title: "Quickstart",
        Icon: RocketLaunchIcon,
      },
    },
    children: [
      {
        url: "/quickstart/core-concepts",
        metadata: {
          title: "Core Concepts",
          description:
            "Understand the core concepts of task queues, TacoQ, and how everything fits together at a basic level.",
          badge: {
            text: "Quickstart",
          },
          sidebar: {
            title: "Core Concepts",
            Icon: AcademicCapIcon,
          },
        },
        content: [
          { type: HeadingTypes.H1, name: "What is TacoQ?" },
          { type: HeadingTypes.H1, name: "Core Concepts: Rapid-fire Overview" },
          { type: HeadingTypes.H2, name: "Tasks" },
          { type: HeadingTypes.H2, name: "Message Broker" },
          { type: HeadingTypes.H2, name: "Workers" },
          { type: HeadingTypes.H2, name: "Publishers" },
          { type: HeadingTypes.H2, name: "Relay" },
          { type: HeadingTypes.H1, name: "What makes TacoQ different?" },
        ],
      },
      {
        url: "/quickstart/setup",
        metadata: {
          title: "Setup",
          description:
            "Get TacoQ up and running on your project using Docker and the Python SDK.",
          badge: {
            text: "Quickstart",
          },
          sidebar: {
            title: "Setup",
            Icon: ArrowDownTrayIcon,
          },
        },
        content: [
          { type: HeadingTypes.H1, name: "Prerequisites" },
          { type: HeadingTypes.H1, name: "Infrastructure" },
          { type: HeadingTypes.H1, name: "Client" },
          { type: HeadingTypes.H2, name: "Worker" },
          { type: HeadingTypes.H2, name: "PublisherClient" },
          { type: HeadingTypes.H2, name: "RelayClient" },
        ],
      },
    ],
  },
  // Technical Reference
  {
    metadata: {
      title: "Technical Reference",
      description: "Learn about the technical details of TacoQ.",
      sidebar: {
        title: "Technical Reference",
        Icon: BookOpenIcon,
      },
    },
    children: [
      {
        url: "/technical-reference/system-architecture",
        metadata: {
          title: "System Architecture",
          description:
            "Learn how services interact with each other and why they are structured the way they are.",
          badge: {
            text: "Technical Reference",
          },
          sidebar: {
            title: "System Architecture",
            Icon: BuildingLibraryIcon,
          },
        },
        content: [
          { type: HeadingTypes.H1, name: "System Services" },
          { type: HeadingTypes.H2, name: "Broker" },
          { type: HeadingTypes.H3, name: "Broker Responsabilities" },
          { type: HeadingTypes.H3, name: "RabbitMQ Implementation Details" },
          { type: HeadingTypes.H2, name: "Database" },
          { type: HeadingTypes.H2, name: "Relay" },
          { type: HeadingTypes.H3, name: "1. Task Update Consumer" },
          { type: HeadingTypes.H3, name: "2. Data Retrieval" },
          { type: HeadingTypes.H3, name: "3. Cleanup" },
          { type: HeadingTypes.H3, name: "4. Replication" },
          { type: HeadingTypes.H1, name: "User Services" },
          { type: HeadingTypes.H2, name: "Worker" },
          { type: HeadingTypes.H2, name: "Publisher Client" },
          { type: HeadingTypes.H2, name: "Relay Client" },
        ],
      },
      {
        url: "/technical-reference/relay-endpoints",
        metadata: {
          title: "Relay Endpoints",
          description:
            "Discover how to interact with the relay endpoints from any language, even ones without a dedicated SDK.",
          badge: {
            text: "Technical Reference",
          },
          sidebar: {
            title: "Relay Endpoints",
            Icon: CpuChipIcon,
          },
        },
        content: [],
      },
      {
        url: "/technical-reference/versioning",
        metadata: {
          title: "Versioning",
          description:
            "Learn how TacoQ handles versioning for images and libraries.",
          badge: {
            text: "Technical Reference",
          },
          sidebar: {
            title: "Versioning",
            Icon: AdjustmentsVerticalIcon,
          },
        },
        content: [
          { type: HeadingTypes.H1, name: "Semantic Versioning" },
          { type: HeadingTypes.H1, name: "SDK and Image Lockstep Releases" },
          { type: HeadingTypes.H1, name: "Task Object" },
          { type: HeadingTypes.H1, name: "Documentation" },
        ],
      },
      {
        url: "/technical-reference/benchmarks",
        metadata: {
          title: "Benchmarks",
          description:
            "Learn how TacoQ compares to other task queues in terms of performance and scalability.",
          badge: {
            text: "Technical Reference",
          },
          sidebar: {
            title: "Benchmarks",
            Icon: ChartBarIcon,
          },
        },
      },

      // {
      //   metadata: {
      //     title: "SDKs",
      //     description: "Reference for each of the available SDKs.",
      //     badge: {
      //       text: "Technical Reference",
      //     },
      //     sidebar: {
      //       title: "SDKs",
      //       Icon: ComputerDesktopIcon,
      //     },
      //   },
      //   children: [
      //     {
      //       url: "/technical-reference/sdks/python",
      //       metadata: {
      //         title: "Python SDK",
      //         description: "Reference for the Python SDK.",
      //         badge: {
      //           text: "Technical Reference",
      //         },
      //         sidebar: {
      //           title: "Python SDK",
      //           Icon: CodeBracketIcon,
      //         },
      //       },
      //     },
      //   ],
      // },
    ],
  },
  // Guides
  {
    metadata: {
      title: "Guides",
      description: "Learn how to perform common tasks using TacoQ.",
      sidebar: {
        title: "Guides",
        Icon: PencilSquareIcon,
      },
    },
    children: [
      {
        url: "/guides/task-versioning",
        metadata: {
          title: "Task Encoding & Versioning",
          description:
            "Learn how to serialize your tasks' input and output data as well as version your tasks.",
          badge: {
            text: "Guides",
          },
          sidebar: {
            title: "Task Encoding & Versioning",
            Icon: ArrowUpIcon,
          },
        },
      },
      {
        url: "/guides/same-app-worker-pattern",
        metadata: {
          title: "Same-app Worker Pattern",
          description:
            "Learn how to set up a worker and publisher in the same application to make your life easier.",
          badge: {
            text: "Guides",
          },
          sidebar: {
            title: "Same-app Worker Pattern",
            Icon: ChevronDoubleRightIcon,
          },
        },
      },
      {
        url: "/guides/maximizing-performance",
        metadata: {
          title: "Maximizing Performance",
          description:
            "Understand how to get the best performance out of TacoQ.",
          badge: {
            text: "Guides",
          },
          sidebar: {
            title: "Maximizing Performance",
            Icon: BoltIcon,
          },
        },
      },
      {
        url: "/guides/horizontal-scaling",
        metadata: {
          title: "Scaling TacoQ",
          description:
            "Learn how to scale your TacoQ application horizontally on the cloud with multiple workers, auto-scaling, distributed Postgres, replicated relays, and more.",
          badge: {
            text: "Guides",
          },
          sidebar: {
            title: "Horizontal Scaling",
            Icon: PresentationChartLineIcon,
          },
        },
      },
      {
        url: "/guides/api-integration",
        metadata: {
          title: "API Integration",
          description:
            "Learn how to publish and fetch tasks without an SDK by using the relay endpoints.",
          badge: {
            text: "Guides",
          },
          sidebar: {
            title: "API Integration",
            Icon: LinkIcon,
          },
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
  /** Array of pages that are the breadcrumbs for the current page */
  breadcrumbs: Page[];
  /** Previous page in navigation sequence, null if none */
  previousPage: Page | null;
  /** Next page in navigation sequence, null if none */
  nextPage: Page | null;
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

/**
 * Gets a flattened array of pages with URLs in navigation order
 * @param pages - Array of pages to flatten
 * @returns Array of pages with URLs in navigation order
 */
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

  /**
   * Memoized breadcrumbs for the current page
   */
  const breadcrumbs = useMemo(() => {
    // Find the parent titles of the current page
    const parentTitles = findPageAndParents(pages, pathname);
    // Find the pages that match the parent titles
    return (
      parentTitles
        .map((title) => findPageByTitle(pages, title))
        // Filter out any null values
        .filter((page): page is Page => !!page)
    );
  }, [pathname]);

  /**
   * Memoized previous and next pages based on current pathname
   */
  const { previousPage, nextPage } = useMemo(() => {
    const flattenedPages = getFlattenedPages(pages);
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
