export type MetadataJson = {
  title: string;
  description: string;
  icon: string;
  index: number;
};

export type HeadingType = "h1" | "h2" | "h3" | "h4" | "h5" | "h6";

export type ContentRow = {
  title: string;
  type: string;
};

export type Page = {
  url?: string;
  metadata: MetadataJson;
  rawContent?: string;
  contentRows?: ContentRow[];
  children?: Page[];
};

export type PageTree = {
  children: Page[];
};
