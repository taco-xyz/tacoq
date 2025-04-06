"use client";

// React Imports
import {
  FC,
  createContext,
  useContext,
  useState,
  useCallback,
  useMemo,
  useEffect,
  PropsWithChildren,
} from "react";

// Next Imports
import { usePathname } from "next/navigation";

// Type imports
import type { PageTree } from "@/types/PageTree";
import type { PageTreeElement } from "@/types/page-tree-element/PageTreeElement";
import type { Folder } from "@/types/page-tree-element/Folder";
//import type { Anchor } from "@/types/Anchor";
import { FooterContent, Status } from "@/types/FooterContent";

// Data imports
import pageTreeJson from "@/page-tree.json";

const pageTree = pageTreeJson as PageTree;

const footerContent: FooterContent = {
  linkGroups: [
    {
      groupName: "Frameworks",
      links: [
        {
          linkName: "TacoQ",
          url: "https://github.com/taco-xyz/tacoq",
          status: Status.COMPLETED,
        },
        { linkName: "TacoDocs", status: Status.WORK_IN_PROGRESS },
        { linkName: "TacoFlow", status: Status.SOON },
        { linkName: "TacoBI", status: Status.SOON },
        { linkName: "TacoCI", status: Status.SOON },
      ],
    },
    {
      groupName: "Taco Plus",
      links: [
        { linkName: "Docs Templates", status: Status.WORK_IN_PROGRESS },
        { linkName: "Early Access", status: Status.SOON },
        { linkName: "Priority Support", status: Status.SOON },
        { linkName: "BI Templates", status: Status.SOON },
      ],
    },
    {
      groupName: "Community",
      links: [
        {
          linkName: "Discord",
          url: "https://discord.gg/NXwBEtZSUq",
          status: Status.COMPLETED,
        },
        {
          linkName: "Github",
          url: "https://github.com/taco-xyz/tacoq",
          status: Status.COMPLETED,
        },
      ],
    },
  ],
};

/**
 * Context for managing the page navigation tree state
 */
interface PageTreeContextType {
  /** Checks if a page tree element is a folder */
  isFolder: (element: PageTreeElement) => element is Folder;
  /** Checks if a folder is currently expanded */
  isFolderExpanded: (folderTitle: string) => boolean;
  /** Expands a folder in the navigation tree */
  expandFolder: (folderTitle: string) => void;
  /** Collapses a folder in the navigation tree */
  collapseFolder: (folderTitle: string) => void;
  /** Retrieves a page tree element by its title */
  getElementByTitle: (elementTitle: string) => PageTreeElement | null;
  /** Title of the currently active page tree element */
  currentElementTitle: string | null;
  /** Array of page tree element titles that are currently visible */
  visibleElementsTitles: string[];
  /** Nested array of all page tree elements in the navigation tree */
  pageTreeElements: PageTreeElement[];
  /** Array of anchor links */
  // anchors: Anchor[];
  /** Footer content */
  footerContent: FooterContent;
  /** Array of page tree element titles that are the breadcrumbs for the current page */
  breadcrumbs: (PageTreeElement | null)[];
  /** Previous page tree element in navigation sequence, null if none */
  previousElement: PageTreeElement | null;
  /** Next page tree element in navigation sequence, null if none */
  nextElement: PageTreeElement | null;
}

const PageTreeContext = createContext<PageTreeContextType | null>(null);

// Helper functions
/**
 * Type guard to determine if a PageTreeElement is a Folder
 * @param element - The element to check
 * @returns True if the element has a children property, indicating it's a Folder
 */
const isFolder = (element: PageTreeElement): element is Folder => {
  return "children" in element;
};

/**
 * Gets an array of visible element titles based on which folders are expanded
 * @param pageTree - Array of page tree elements to traverse
 * @param expandedFoldersTitles - Set of folder titles that are currently expanded
 * @returns Array of titles for all visible elements in the tree
 */
function getVisibleElements(
  pageTree: PageTreeElement[],
  expandedFoldersTitles: Set<string>,
): string[] {
  const visibleElements: string[] = [];
  function traverse(element: PageTreeElement) {
    visibleElements.push(element.metadata.title);

    if (
      !isFolder(element) ||
      !expandedFoldersTitles.has(element.metadata.title)
    )
      return;

    element.children.forEach(traverse);
  }
  pageTree.forEach(traverse);
  return visibleElements;
}

/**
 * Recursively searches for a page tree element by its title
 * @param elements - Array of elements to search through
 * @param title - Title of the element to find
 * @returns The found element or null if not found
 */
function findElementByTitle(
  elements: PageTreeElement[],
  title: string,
): PageTreeElement | null {
  for (const element of elements) {
    if (element.metadata.title === title) return element;

    if (!isFolder(element)) continue;

    const found = findElementByTitle(element.children, title);
    if (found) return found;
  }
  return null;
}

/**
 * Finds the breadcrumb path to a page with the given URL
 * @param elements - Array of elements to search through
 * @param targetUrl - URL of the page to find
 * @param breadcrumbs - Accumulator for recursive breadcrumb collection
 * @returns Array of titles representing the path to the target page
 */
function findBreadcrumbs(
  elements: PageTreeElement[],
  targetUrl: string,
  breadcrumbs: string[] = [],
): string[] {
  for (const element of elements) {
    if (!isFolder(element)) {
      if (element.url === targetUrl) {
        return [...breadcrumbs, element.metadata.title];
      }
    } else {
      const found = findBreadcrumbs(element.children, targetUrl, [
        ...breadcrumbs,
        element.metadata.title,
      ]);
      if (found.length > 0) return found;
    }
  }
  return [];
}

/**
 * Flattens the page tree into a single array of elements
 * @param elements - Array of elements to flatten
 * @returns Flattened array containing all elements in the tree
 */
function getFlattenedElements(elements: PageTreeElement[]): PageTreeElement[] {
  const flattened: PageTreeElement[] = [];
  function traverse(element: PageTreeElement) {
    flattened.push(element);

    if (!isFolder(element)) return;

    element.children.forEach(traverse);
  }
  elements.forEach(traverse);
  return flattened;
}

export const PageTreeProvider: FC<PropsWithChildren> = ({ children }) => {
  const pathname = usePathname();

  const [expandedFoldersTitles, setExpandedFoldersTitles] = useState<
    Set<string>
  >(() => {
    const breadcrumbs = findBreadcrumbs(pageTree.children, pathname);
    // Remove the last breadcrumb because it's the current page (not a folder)
    return new Set(breadcrumbs.slice(0, -1));
  });

  /**
   * Retrieves the title of the element that matches the current pathname
   */
  const currentElementTitle = useMemo(() => {
    const breadcrumbs = findBreadcrumbs(pageTree.children, pathname);
    // The last breadcrumb is the current page
    return breadcrumbs[breadcrumbs.length - 1];
  }, [pathname]);

  /**
   * Retrieves an array of visible element titles based on which folders are expanded
   */
  const visibleElementsTitles = useMemo(
    () => getVisibleElements(pageTree.children, expandedFoldersTitles),
    [expandedFoldersTitles],
  );

  /**
   * Expands a folder in the navigation tree
   * @param folderTitle - Title of the folder to expand
   */
  const expandFolder = useCallback((folderTitle: string) => {
    setExpandedFoldersTitles((prev) => new Set([...prev, folderTitle]));
  }, []);

  /**
   * Collapses a folder in the navigation tree
   * @param folderTitle - Title of the folder to collapse
   */
  const collapseFolder = useCallback((folderTitle: string) => {
    setExpandedFoldersTitles((prev) => {
      const next = new Set(prev);
      next.delete(folderTitle);
      return next;
    });
  }, []);

  /**
   * Checks if a folder is currently expanded
   * @param folderTitle - Title of the folder to check
   * @returns True if the folder is expanded, false otherwise
   */
  const isFolderExpanded = useCallback(
    (folderTitle: string) => expandedFoldersTitles.has(folderTitle),
    [expandedFoldersTitles],
  );

  /**
   * Retrieves a page tree element by its title
   * @param elementTitle - Title of the element to find
   * @returns The found element or null if not found
   */
  const getElementByTitle = useCallback(
    (elementTitle: string) =>
      findElementByTitle(pageTree.children, elementTitle),
    [],
  );

  /**
   * Retrieves the breadcrumb path to the current page
   */
  const breadcrumbs = useMemo(() => {
    const breadcrumbs = findBreadcrumbs(pageTree.children, pathname);
    return breadcrumbs.map((title) =>
      findElementByTitle(pageTree.children, title),
    );
  }, [pathname]);

  /**
   * Retrieves the previous and next page tree elements in the navigation sequence or null if not found
   */
  const { previousElement, nextElement } = useMemo(() => {
    const flattenedPages = getFlattenedElements(pageTree.children).filter(
      (element) => !isFolder(element),
    );
    const currentIndex = flattenedPages.findIndex(
      (element) => !isFolder(element) && element.url === pathname,
    );

    // To avoid returning -1 if the targetUrl is not found
    if (currentIndex < 0) return { previousElement: null, nextElement: null };

    const previousElement =
      currentIndex > 0 ? flattenedPages[currentIndex - 1] : null;
    const nextElement =
      currentIndex < flattenedPages.length - 1
        ? flattenedPages[currentIndex + 1]
        : null;

    return {
      previousElement,
      nextElement,
    };
  }, [pathname]);

  /**
   * Expands the parent folders of the current page when the pathname changes
   * This is to ensure that when traveling to documentation pages through exterior links the parent folders
   * to the current page are expanded
   */
  useEffect(() => {
    const parentPages = findBreadcrumbs(pageTree.children, pathname);
    // Remove the last breadcrumb because it's the current page (not a folder)
    setExpandedFoldersTitles(
      (prev) => new Set([...prev, ...parentPages.slice(0, -1)]),
    );
  }, [pathname]);

  return (
    <PageTreeContext.Provider
      value={{
        isFolder,
        isFolderExpanded,
        expandFolder,
        collapseFolder,
        getElementByTitle,
        currentElementTitle,
        visibleElementsTitles,
        pageTreeElements: pageTree.children,
        footerContent,
        breadcrumbs,
        previousElement,
        nextElement,
        //anchors
      }}
    >
      {children}
    </PageTreeContext.Provider>
  );
};

export function usePageTree() {
  const context = useContext(PageTreeContext);
  if (!context) {
    throw new Error("usePageTree must be used within PageTreeProvider");
  }
  return context;
}
