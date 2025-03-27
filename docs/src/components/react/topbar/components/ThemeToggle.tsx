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
      className="group custom-tab-outline-offset-2 relative flex flex-shrink-0 cursor-pointer flex-row items-center gap-x-4 rounded-full p-2 ring-1 ring-zinc-200 transition-all duration-150 ease-in-out ring-inset hover:ring-zinc-200/90 dark:ring-zinc-800 hover:dark:ring-zinc-700"
    >
      <div className="flex-shrink-0 text-zinc-500 transition-all duration-150 ease-in-out group-hover:text-zinc-400 dark:text-white/70 dark:group-hover:text-white/90">
        <Sun className="size-5" />
      </div>
      <div className="flex-shrink-0 text-zinc-500 transition-all duration-150 ease-in-out group-hover:text-zinc-400 dark:text-white/70 dark:group-hover:text-white/90">
        <MoonStar className="size-5" />
      </div>
      <div className="absolute left-1 -z-1 size-7 rounded-full bg-zinc-200 transition-all duration-150 ease-in-out group-hover:bg-zinc-200/90 dark:translate-x-9 dark:bg-zinc-800 dark:group-hover:bg-zinc-700" />
    </button>
  );
}
