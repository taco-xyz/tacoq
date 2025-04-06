// React Imports
import { FC } from "react";

// Component Imports
import { Logo } from "../Logo";
import { ThemeToggle } from "./components/ThemeToggle";
import { GitHubLink } from "./components/GitHubLink";
import { DesktopSearchButton } from "./components/search/components/DesktopSearchButton";
import { SearchDialog } from "./components/search/components/SearchPortal";
import { MobileSearchButton } from "./components/search/components/MobileSearchButton";
import { MobileSidebarPortal } from "../sidebar/MobileSidebarPortal";
import { MobileSidebarButton } from "../sidebar/MobileSidebarButton";
import { Breadcrumbs } from "../docs-page-layout/breadcrumbs/Breadcrumbs";

// Context Imports
import { MobileSidebarModalProvider } from "../sidebar/context/MobileSidebarModalContext";
import { SearchModalProvider } from "./components/search/context/SearchModalContext";

/**
 * TopBar component that serves as the common header for the application
 * Contains:
 * - Logo
 * - Search
 * - Theme toggle
 * - GitHub link
 */
export const TopBar: FC = () => {
  return (
    <div className="flex h-fit w-full flex-col items-center justify-center overflow-hidden border-b border-b-zinc-200 bg-white/50 backdrop-blur-lg transition-all duration-150 ease-in-out md:h-[80px] md:flex-row md:backdrop-blur-md dark:border-b-white/10 dark:bg-zinc-950/50">
      <div className="relative flex w-full max-w-(--breakpoint-2xl) flex-row items-center justify-center px-8 py-5">
        <div className="z-1 flex w-full flex-row items-center justify-between">
          <Logo />

          <div className="relative inset-0 mx-auto my-auto hidden h-fit w-[500px] justify-center lg:flex">
            {/* Decorative Background gradient */}
            <div className="pointer-events-none absolute -bottom-5 z-[-1] h-full overflow-hidden">
              <div className="pointer-events-none h-36 w-[100rem] origin-center bg-radial from-zinc-400 from-0% via-zinc-400/50 via-15% to-transparent to-50% opacity-25 dark:from-white dark:via-white/50 dark:opacity-15" />
            </div>

            {/* Desktop Search */}
            <SearchModalProvider>
              <DesktopSearchButton />
              <SearchDialog />
            </SearchModalProvider>
          </div>

          <div className="flex flex-row items-center gap-x-6 md:gap-x-8">
            <div className="relative inset-0 mx-auto my-auto flex h-fit w-fit justify-center sm:w-[300px] lg:hidden">
              {/* Decorative Background gradient */}
              <div className="pointer-events-none absolute -bottom-5 z-[-1] hidden h-full overflow-hidden md:block">
                <div className="pointer-events-none h-36 origin-center bg-radial from-zinc-400 from-0% via-zinc-400/50 via-15% to-transparent to-50% opacity-30 md:w-[55rem] dark:from-white dark:via-white/50 dark:opacity-20" />
              </div>

              {/* Tablet and Mobile Search */}
              <SearchModalProvider>
                <div className="hidden w-full sm:block">
                  <DesktopSearchButton />
                </div>
                <div className="flex items-center justify-center sm:hidden">
                  <MobileSearchButton />
                </div>
                <SearchDialog />
              </SearchModalProvider>
            </div>

            {/* Theme Toggle and GitHub Link */}
            <div className="hidden flex-row items-center justify-end gap-x-8 md:flex lg:w-[100px]">
              <ThemeToggle />
              <GitHubLink />
            </div>

            <div className="flex items-center justify-center md:hidden">
              <MobileSidebarModalProvider>
                <MobileSidebarButton />
                <MobileSidebarPortal />
              </MobileSidebarModalProvider>
            </div>
          </div>
        </div>
      </div>
      {/* Mobile and Tablet Divider */}
      <div className="h-[1px] w-full px-8 md:hidden">
        <div className="border-b border-b-zinc-200 dark:border-b-white/5"></div>
      </div>
      {/* Mobile and Tablet Breadcrumbs */}
      <div className="relative flex w-full items-center justify-center md:hidden">
        <Breadcrumbs />
        {/* Decorative Background gradient */}
        <div className="pointer-events-none absolute -bottom-7 z-[-1] flex h-[50px] w-full items-center justify-center overflow-hidden md:hidden">
          <div className="pointer-events-none h-20 w-[40rem] origin-center bg-radial from-zinc-400 from-0% via-zinc-400/50 via-15% to-transparent to-50% opacity-10 dark:from-white dark:via-white/50 dark:opacity-4" />
        </div>
      </div>
    </div>
  );
};
