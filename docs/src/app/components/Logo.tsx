// Next Imports
import Link from "next/link";

/**
 * Logo component that displays the application logo and title
 * Links to the home page when clicked
 */
export default function Logo() {
  return (
    <Link
      href="/"
      className="flex flex-row items-center space-x-0.5 cursor-pointer custom-tab-outline-offset-6 rounded-sm transition-all ease-in-out duration-150"
    >
      <div className="size-[27px] shadow-xl rounded-sm dark:shadow-white/30 shadow-zinc-400/20 dark:bg-white bg-zinc-400 transition-all ease-in-out duration-150">
        <svg
          width="100"
          height="100"
          viewBox="0 0 100 100"
          className="w-full h-full"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M35.6695 87L42.1195 34.6C42.3529 32.8 42.9195 31.3667 43.8195 30.3C44.7195 29.2333 46.0529 28.5833 47.8195 28.35H24.5195L26.2695 14.3H78.8695L77.1195 28.35H58.8195L51.6195 87H35.6695Z"
            className="dark:fill-zinc-900 fill-white transition-[fill] ease-in-out duration-150"
          />
        </svg>
      </div>
      <p className="dark:text-white text-zinc-700 font-black font-mono text-3xl transition-all ease-in-out duration-150">
        .doc
      </p>
    </Link>
  );
}
