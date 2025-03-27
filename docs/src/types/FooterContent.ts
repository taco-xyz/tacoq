export enum Status {
  SOON = "Soon",
  WORK_IN_PROGRESS = "WIP",
  COMPLETED = "Completed",
}

export interface FooterLink {
  linkName: string;
  status: Status;
}

export interface CompletedFooterLink extends FooterLink {
  status: Status.COMPLETED;
  url: string;
}

export interface SoonFooterLink extends FooterLink {
  status: Status.SOON | Status.WORK_IN_PROGRESS;
}

export interface FooterLinkGroup {
  groupName: string;
  links: (CompletedFooterLink | SoonFooterLink)[];
}

export interface FooterContent {
  linkGroups: FooterLinkGroup[];
}
