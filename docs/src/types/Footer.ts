export type Status = "Soon" | "WIP" | "Completed";

export interface FooterLink {
  linkName: string;
  status: Status;
}

export interface CompletedFooterLink extends FooterLink {
  status: "Completed";
  url: string;
}

export interface SoonFooterLink extends FooterLink {
  status: "Soon" | "WIP";
}

export interface FooterLinkGroup {
  groupName: string;
  links: (CompletedFooterLink | SoonFooterLink)[];
}

export interface Footer {
  linkGroups: FooterLinkGroup[];
}
