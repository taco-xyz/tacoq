export type MetadataJson = {
  title: string;
  description: string;
  icon: string;
  index: number;
};

export type HeaderType = "h1" | "h2" | "h3" | "h4" | "h5" | "h6";

export type Header = {
  title: string;
  type: HeaderType;
};

export type Page = {
  url?: string;
  metadata: MetadataJson;
  rawContent?: string;
  headers?: Header[];
  children?: Page[];
};

export type PageTree = {
  children: Page[];
};
