// Component Imports
import Logo from "../Logo";
import ThemeToggle from "./components/ThemeToggle";
import GitHubLink from "./components/GitHubLink";
import DesktopSearchButton from "./components/search/components/DesktopSearchButton";
import SearchDialog from "./components/search/components/SearchPortal";
import MobileSearchButton from "./components/search/components/MobileSearchButton";
import MobileSidebarPortal from "../sidebar/MobileSidebarPortal";
import MobileSidebarButton from "../sidebar/MobileSidebarButton";
import Breadcrumbs from "../docs-page-layout/breadcrumbs/Breadcrumbs";

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
export function TopBar() {
  return (
    <div className="items-center md:h-[80px] h-fit md:backdrop-blur-md backdrop-blur-lg overflow-hidden justify-center flex md:flex-row flex-col w-full dark:border-b-white/10 border-b-zinc-200 border-b dark:bg-zinc-950/50 bg-white/50 transition-all ease-in-out duration-150">
      <div className="flex flex-row justify-center items-center w-full max-w-(--breakpoint-2xl) py-5 px-8 relative">
        <div className="flex flex-row z-1 w-full items-center justify-between ">
          <Logo />

          <div className="h-fit w-[500px] my-auto mx-auto inset-0 hidden lg:flex relative justify-center">
            {/* Decorative Background gradient */}
            <div className="absolute overflow-hidden h-full -bottom-5 z-[-1] pointer-events-none">
              <div className="bg-radial origin-center h-36 w-[100rem] opacity-25 dark:opacity-15 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% pointer-events-none" />
            </div>

            {/* Desktop Search */}
            <SearchModalProvider>
              <DesktopSearchButton />
              <SearchDialog />
            </SearchModalProvider>
          </div>

          <div className="flex flex-row items-center md:gap-x-8 gap-x-6">
            <div className="h-fit sm:w-[300px] w-fit my-auto mx-auto inset-0 lg:hidden flex relative justify-center">
              {/* Decorative Background gradient */}
              <div className="absolute overflow-hidden h-full -bottom-5 hidden md:block z-[-1] pointer-events-none">
                <div className="bg-radial origin-center h-36 md:w-[55rem] dark:opacity-20 opacity-30 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% pointer-events-none" />
              </div>

              {/* Tablet and Mobile Search */}
              <SearchModalProvider>
                <div className="hidden sm:block w-full">
                  <DesktopSearchButton />
                </div>
                <div className="sm:hidden flex justify-center items-center">
                  <MobileSearchButton />
                </div>
                <SearchDialog />
              </SearchModalProvider>
            </div>

            {/* Theme Toggle and GitHub Link */}
            <div className="md:flex lg:w-[100px] hidden flex-row items-center justify-end gap-x-8">
              <ThemeToggle />
              <GitHubLink />
            </div>

            <div className="md:hidden flex justify-center items-center">
              <MobileSidebarModalProvider>
                <MobileSidebarButton />
                <MobileSidebarPortal />
              </MobileSidebarModalProvider>
            </div>
          </div>
        </div>
      </div>
      {/* Mobile and Tablet Divider */}
      <div className="w-full h-[1px] px-8 md:hidden">
        <div className="dark:border-b-white/5 border-b-zinc-200 border-b"></div>
      </div>
      {/* Mobile and Tablet Breadcrumbs */}
      <div className="md:hidden w-full relative flex items-center justify-center">
        <Breadcrumbs />
        {/* Decorative Background gradient */}
        <div className="absolute overflow-hidden h-[50px] md:hidden z-[-1] -bottom-7 w-full flex items-center justify-center pointer-events-none">
          <div className="bg-radial origin-center h-20 w-[40rem] dark:opacity-4 opacity-10 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% pointer-events-none" />
        </div>
      </div>
    </div>
  );
}
