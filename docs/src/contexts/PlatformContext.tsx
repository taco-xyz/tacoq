"use client";

// React Imports
import { createContext, useContext, useEffect, useState } from "react";

type PlatformContextType = {
  isMacOS: boolean;
};

const PlatformContext = createContext<PlatformContextType>({ isMacOS: false });

export function PlatformProvider({ children }: { children: React.ReactNode }) {
  const [isMacOS, setIsMacOS] = useState(false);

  useEffect(() => {
    // Check if running on macOS/iOS
    const platform = window.navigator.platform.toLowerCase();
    setIsMacOS(
      platform.includes("mac") ||
        platform.includes("iphone") ||
        platform.includes("ipad"),
    );
  }, []);

  return (
    <PlatformContext.Provider value={{ isMacOS }}>
      {children}
    </PlatformContext.Provider>
  );
}

export const usePlatform = () => useContext(PlatformContext);
