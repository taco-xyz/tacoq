"use client";

// React Imports
import {
  createContext,
  useContext,
  useEffect,
  useState,
  PropsWithChildren,
  FC,
} from "react";

type PlatformContextType = {
  isAppleDevice: boolean;
};

const PlatformContext = createContext<PlatformContextType>({
  isAppleDevice: false,
});

export const PlatformProvider: FC<PropsWithChildren> = ({ children }) => {
  const [isAppleDevice, setIsAppleDevice] = useState(false);

  useEffect(() => {
    const ua = navigator.userAgent.toLowerCase();
    const isApple =
      /macintosh|macintel|macppc|mac68k|mac os x|iphone|ipad|ipod/.test(ua);
    setIsAppleDevice(isApple);
  }, []);

  return (
    <PlatformContext.Provider value={{ isAppleDevice }}>
      {children}
    </PlatformContext.Provider>
  );
};

export function usePlatform() {
  const context = useContext(PlatformContext);
  if (!context) {
    throw new Error("usePlatform must be used within a PlatformProvider");
  }
  return context;
}
