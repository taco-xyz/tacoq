"use client";

import {
  createContext,
  useContext,
  useEffect,
  useState,
  PropsWithChildren,
} from "react";

type PlatformContextType = {
  isAppleDevice: boolean;
};

const PlatformContext = createContext<PlatformContextType>({
  isAppleDevice: false,
});

export function PlatformProvider({ children }: PropsWithChildren) {
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
}

export const usePlatform = () => useContext(PlatformContext);
