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
      <div>
        <Image
          src="/TacoDivisionLogoDark.svg"
          alt="Logo"
          width={40}
          height={40}
          className="block dark:hidden"
        />
        <Image
          src="/TacoDivisionLogoLight.svg"
          alt="Logo"
          width={40}
          height={40}
          className="hidden dark:block"
        />
      </div>
    </Link>
  );
}
