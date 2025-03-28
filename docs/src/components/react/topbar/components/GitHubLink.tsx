// Next Imports
import Link from "next/link";

// Custom Icon Imports
import { GithubIcon } from "../../icons/social";

/**
 * GitHub link component that links to the repository
 * Opens in a new tab when clicked
 */
export function GitHubLink() {
  return (
    <Link
      href="https://github.com/taco-xyz/tacoq"
      target="_blank"
      className="custom-tab-outline-offset-4 rounded-xs text-zinc-500 transition-all duration-150 ease-in-out hover:text-zinc-400 dark:text-white/70 dark:hover:text-white/80"
    >
      <GithubIcon className="size-5" />
    </Link>
  );
}
