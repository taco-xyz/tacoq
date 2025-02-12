// React Imports
import type { SVGProps } from "react";

export interface FooterLink {
  linkName: string;
  url: string;
}

export interface FooterLinkGroup {
  groupName: string;
  links: FooterLink[];
}

export interface SocialLink {
  Icon: React.ComponentType<SVGProps<SVGSVGElement>>;
  url: string;
}

export default interface FooterContent {
  linkGroups: FooterLinkGroup[];
  socialLinks: SocialLink[];
}
