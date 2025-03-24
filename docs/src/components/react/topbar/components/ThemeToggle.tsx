"use client";

// React Imports
import { useCallback } from "react";

// Lucide Icons
import { Sun, MoonStar } from "lucide-react";

/**
 * Theme toggle component that switches between light and dark modes
 * Uses localStorage to persist theme preference
 */
export default function ThemeToggle() {
  const toggleTheme = useCallback(() => {
    if (document.documentElement.classList.contains("dark")) {
      localStorage.setItem("theme", "light");
      document.documentElement.classList.remove("dark");
    } else {
      localStorage.setItem("theme", "dark");
      document.documentElement.classList.add("dark");
    }
  }, []);

  return (
    <button
      onClick={toggleTheme}
      className="flex relative flex-row gap-x-4 group cursor-pointer items-center p-2 rounded-full ring-1 ring-inset ring-zinc-200 hover:ring-zinc-200/90 dark:ring-zinc-800 hover:dark:ring-zinc-700 flex-shrink-0 transition-all ease-in-out duration-150 custom-tab-outline-offset-2"
    >
      <div className="dark:group-hover:text-white/90 flex-shrink-0 text-zinc-500 group-hover:text-zinc-400 dark:text-white/70 transition-all ease-in-out duration-150">
        <Sun className="size-5" />
      </div>
      <div className="dark:group-hover:text-white/90 flex-shrink-0 text-zinc-500 group-hover:text-zinc-400 dark:text-white/70 transition-all ease-in-out duration-150">
        <MoonStar className="size-5" />
      </div>
      <div className="size-7 rounded-full bg-zinc-200 dark:bg-zinc-800  group-hover:bg-zinc-200/90 dark:group-hover:bg-zinc-700 absolute -z-1 left-1 dark:translate-x-9 transition-all ease-in-out duration-150" />
    </button>
  );
}
