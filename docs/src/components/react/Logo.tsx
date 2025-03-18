// Next Imports
import Link from "next/link";
import Image from "next/image";

/**
 * Logo component that displays the application logo and title
 * Links to the home page when clicked
 */
export default function Logo() {
  return (
    <Link href="/">
      <Image
        src="/TacoQTextLogoLight.svg"
        alt="Logo"
        width={100}
        height={100}
        className="block dark:hidden w-[100px] h-auto"
      />
      <Image
        src="/TacoQTextLogoDark.svg"
        alt="Logo"
        width={100}
        height={100}
        className="hidden dark:block w-[100px] h-auto"
      />
    </Link>
  );
}

/**
 * Logo component that displays the company logo and title
 */
export function DivisionLogo() {
  return (
    <div>
      <Image
        src="/TacoDivisionTextLogoLight.svg"
        alt="Logo"
        width={150}
        height={100}
        className="block dark:hidden w-[150px] h-auto"
      />
      <Image
        src="/TacoDivisionTextLogoDark.svg"
        alt="Logo"
        width={150}
        height={100}
        className="hidden dark:block w-[150px] h-auto"
      />
    </div>
  );
}
