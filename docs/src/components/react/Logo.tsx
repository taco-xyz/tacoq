// React Imports
import { FC } from "react";

// Next Imports
import Link from "next/link";

// Components Imports
import { PriorityImageWithTheme } from "./PriorityImageWithTheme";

/**
 * Logo component that displays the application logo and title
 * Links to the home page when clicked
 */
export const Logo: FC = () => {
  return (
    <Link
      href="/"
      className="custom-tab-outline-offset-6 flex-shrink-0 rounded-xs transition-all duration-150 ease-in-out"
    >
      <PriorityImageWithTheme
        lightSrc="/TacoQTextLogoLight.svg"
        darkSrc="/TacoQTextLogoDark.svg"
        alt="TacoQ Logo"
        width={100}
        height={100}
        lightClassName=""
        darkClassName=""
        commonClassName="w-[100px] h-auto"
      />
    </Link>
  );
}

/**
 * Logo component that displays the company logo and title
 */
export const DivisionLogo: FC = () => {
  return (
    <Link
      href="https://github.com/taco-xyz"
      className="custom-tab-outline-offset-6 flex-shrink-0 rounded-xs transition-all duration-150 ease-in-out"
    >
      <PriorityImageWithTheme
        lightSrc="/TacoDivisionTextLogoLight.svg"
        darkSrc="/TacoDivisionTextLogoDark.svg"
        alt="TacoDivision Logo"
        width={150}
        height={100}
        lightClassName=""
        darkClassName=""
        commonClassName="w-[150px] h-auto"
      />
    </Link>
  );
}
