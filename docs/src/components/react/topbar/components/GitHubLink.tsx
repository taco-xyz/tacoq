// Next Imports
import Link from "next/link";

// Custom Icon Imports
import { GithubIcon } from "../../icons/social";

/**
 * GitHub link component that links to the repository
 * Opens in a new tab when clicked
 */
export default function GitHubLink() {
  return (
    <Link
      href="https://github.com/taco-xyz/tacoq"
      target="_blank"
      className="dark:hover:text-white/80 text-zinc-500 hover:text-zinc-400 dark:text-white/70 transition-all ease-in-out duration-150 rounded-xs custom-tab-outline-offset-4"
    >
      <GithubIcon className="size-5" />
    </Link>
  );
}
