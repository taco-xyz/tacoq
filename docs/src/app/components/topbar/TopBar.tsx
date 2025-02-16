// Component Imports
import Logo from "../Logo";
import Search from "./components/Search";
import ThemeToggle from "./components/ThemeToggle";
import GitHubLink from "./components/GitHubLink";

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
    <div className="items-center backdrop-blur-md justify-center flex w-full dark:border-b-white/10 border-b-zinc-200 border-b dark:bg-zinc-950/50 bg-white/50 transition-all ease-in-out duration-150">
      <div className="flex flex-row justify-center items-center w-full  2xl:max-w-(--breakpoint-2xl) py-5 px-8">
        {/* Decorative Background gradient */}
        <div className="absolute overflow-hidden h-full">
          <div className="bg-radial origin-center h-36 w-[100rem] -bottom-20 opacity-10 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
        </div>

        <div className="flex flex-row z-1 w-full items-center justify-between ">
          <Logo />

          <div className="absolute w-fit h-fit  my-auto mx-auto inset-0">
            <Search />
          </div>

          <div className="flex flex-row items-center space-x-8 ">
            <ThemeToggle />
            <GitHubLink />
          </div>
        </div>
      </div>
    </div>
  );
}
