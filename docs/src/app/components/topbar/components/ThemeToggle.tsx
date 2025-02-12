"use client"

// React Imports
import { useCallback } from "react";

// Heroicons Imports
import { SunIcon, MoonIcon } from "@heroicons/react/24/outline";

/**
 * Theme toggle component that switches between light and dark modes
 * Uses localStorage to persist theme preference
 */
export default function ThemeToggle() {
  const changeToDarkTheme = useCallback(() => {
    localStorage.setItem("theme", "dark");
    document.documentElement.classList.add("dark");
  }, []);

  const changeToLightTheme = useCallback(() => {
    localStorage.setItem("theme", "light");
    document.documentElement.classList.remove("dark");
  }, []);

  return (
    <>
      <button
        onClick={changeToLightTheme}
        className="dark:hover:text-white/80 hidden dark:block text-zinc-500 hover:text-zinc-400 cursor-pointer dark:text-white/70 transition-all ease-in-out duration-150 custom-tab-outline-offset-4 rounded-xs"
      >
        <MoonIcon className="w-5 h-5" />
      </button>
      <button
        className="dark:hover:text-white/80 dark:hidden text-zinc-500 hover:text-zinc-400 cursor-pointer dark:text-white/70 transition-all ease-in-out duration-150 custom-tab-outline-offset-4 rounded-xs"
        onClick={changeToDarkTheme}
      >
        <SunIcon className="w-5 h-5" />
      </button>
    </>
  );
}
