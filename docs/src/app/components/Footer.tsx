import Link from "next/link";
import Logo from "./topbar/components/Logo";
import { SVGProps } from "react";

interface FooterLink {
  linkName: string;
  url: string;
}

interface FooterLinkGroup {
  groupName: string;
  links: FooterLink[];
}

interface SocialLink {
  Icon?: React.ComponentType<SVGProps<SVGSVGElement>>;
  url: string;
}

interface FooterInfo {
  linkGroups?: FooterLinkGroup[];
  socialLinks?: SocialLink[];
}

export function Footer({ footerInfo }: { footerInfo: FooterInfo }) {
  return (
    <div className="items-center justify-center absolute flex flex-col w-full dark:border-t-white/10 border-t-zinc-200 border-t dark:bg-zinc-950/50 bg-white/50 transition-all ease-in-out duration-150">
      <div className="flex flex-row relative items-start overflow-hidden w-full max-w-(--breakpoint-2xl) py-20  px-8 gap-x-20 justify-between border-b border-zinc-200 dark:border-white/10">
        <Logo />

        {footerInfo.linkGroups && footerInfo.linkGroups.length > 0 && (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-y-8 gap-x-20 w-fit">
            {footerInfo.linkGroups.map((group, idx) => (
              <div
                key={idx}
                className="flex flex-col gap-y-3 whitespace-nowrap text-sm"
              >
                <h3 className="font-medium text-zinc-700 dark:text-white transition-colors ease-in-out duration-150">
                  {group.groupName}
                </h3>
                <div className="flex flex-col gap-y-2">
                  {group.links.map((link, linkIdx) => (
                    <Link
                      key={linkIdx}
                      href={link.url}
                      className="text-zinc-500 dark:text-white/50 hover:text-zinc-700 dark:hover:text-white/70 transition-all ease-in-out duration-150 custom-tab-outline-offset-2 rounded-sm"
                    >
                      {link.linkName}
                    </Link>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        {footerInfo.socialLinks && footerInfo.socialLinks.length > 0 && (
          <div className="flex flex-row space-x-4 cursor-pointer">
            {footerInfo.socialLinks.map((social, idx) => (
              <Link
                key={idx}
                href={social.url}
                className="dark:hover:text-white/80 text-zinc-500 hover:text-zinc-400 dark:text-white/70 transition-all ease-in-out duration-150 rounded-xs custom-tab-outline-offset-4"
              >
                {social.Icon && <social.Icon className="size-5" />}
              </Link>
            ))}
          </div>
        )}
        {/* Decorative Background gradient */}
        <div className="absolute overflow-hidden h-full -bottom-50">
          <div className="bg-radial origin-center h-36 w-[100rem] opacity-10 dark:opacity-4 dark:from-white from-zinc-400 dark:via-white/50 via-zinc-400/50 from-0% via-15% to-transparent to-50% " />
        </div>
      </div>
      <div className="flex flex-row items-center justify-center w-full max-w-(--breakpoint-2xl) py-8 px-8">
        <p className="text-zinc-400 dark:text-white/40 text-xs">
          &copy; {new Date().getFullYear()} Taco. All rights reserved.
        </p>
      </div>
    </div>
  );
}
