// Component Imports
import Logo from "../Logo";
import ThemeToggle from "./components/ThemeToggle";
import GitHubLink from "./components/GitHubLink";
import { DesktopSearchButton } from "./components/search/components/DesktopSearchButton";
import { SearchDialog } from "./components/search/components/SearchPortal";
import { SearchProvider } from "./components/search/context/SearchContext";
import { MobileSearchButton } from "./components/search/components/MobileSearchButton";

// Heroicons Imports
import { Bars3CenterLeftIcon } from "@heroicons/react/24/outline";

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
    <div className="items-center h-[80px] backdrop-blur-md overflow-hidden justify-center flex w-full dark:border-b-white/10 border-b-zinc-200 border-b dark:bg-zinc-950/50 bg-white/50 transition-all ease-in-out duration-150">
      <div className="flex flex-row justify-center items-center w-full max-w-(--breakpoint-2xl) py-5 px-8 relative">
        <div className="flex flex-row z-1 w-full items-center justify-between ">
          <Logo />

          <div className="h-fit w-[500px] my-auto mx-auto inset-0 hidden lg:flex relative justify-center">
            {/* Decorative Background gradient */}
            <div className="absolute overflow-hidden h-full -bottom-5">
              <div className="bg-radial origin-center h-36 w-[100rem] opacity-25 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
            </div>

            {/* Desktop Search */}
            <SearchProvider>
              <DesktopSearchButton />
              <SearchDialog />
            </SearchProvider>
          </div>

          <div className="flex flex-row items-center sm:gap-x-8 gap-x-6">
            <div className="h-fit sm:w-[300px] md:w-[400px] w-fit my-auto mx-auto inset-0 lg:hidden flex relative justify-center">
              {/* Decorative Background gradient */}
              <div className="absolute overflow-hidden h-full -bottom-5 hidden sm:block">
                <div className="bg-radial origin-center h-36 md:w-[55rem] w-[50rem] sm:opacity-20 opacity-100 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
              </div>

              {/* Tablet and Mobile Search */}
              <SearchProvider>
                <div className="hidden sm:block w-full">
                  <DesktopSearchButton />
                </div>
                <div className="sm:hidden flex justify-center items-center">
                  <MobileSearchButton />
                </div>
                <SearchDialog />
              </SearchProvider>
            </div>

            {/* Theme Toggle and GitHub Link */}
            <div className="lg:flex hidden flex-row items-center gap-x-8 ">
              <ThemeToggle />
              <GitHubLink />
            </div>

            <button className="md:hidden dark:hover:text-white/80 w-fit h-fit text-zinc-500 hover:text-zinc-400 dark:text-white/70 transition-all ease-in-out duration-150 rounded-xs custom-tab-outline-offset-4">
              <Bars3CenterLeftIcon className="size-5" />
            </button>
          </div>
        </div>
        {/* Decorative Background gradient */}
        <div className="absolute overflow-hidden h-full -bottom-9 sm:hidden">
          <div className="bg-radial origin-center h-20 w-[40rem] opacity-5 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
        </div>
      </div>
    </div>
  );
}
