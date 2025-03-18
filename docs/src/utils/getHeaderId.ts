import type { Header } from "@/types/PageTree";

function getHeaderId(header: Header) {
  return header.type + "-" + header.title.toLowerCase().replace(/\s+/g, "-");
}

export default getHeaderId;
