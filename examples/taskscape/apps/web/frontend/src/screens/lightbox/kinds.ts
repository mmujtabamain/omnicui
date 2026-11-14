// What kind of attachment a row is, and how its name splits for the rename prompt.
import type { Attachment } from "../../api/types";

export type FileKind = "image" | "video" | "audio" | "pdf" | "text" | "file" | "link";

const KIND_BY_EXTENSION: Record<string, FileKind> = {
  png: "image", jpg: "image", jpeg: "image", gif: "image", webp: "image", svg: "image", bmp: "image", avif: "image",
  mp4: "video", webm: "video", mov: "video", m4v: "video",
  mp3: "audio", wav: "audio", ogg: "audio", m4a: "audio", flac: "audio", aac: "audio",
  pdf: "pdf",
  txt: "text", md: "text", csv: "text", json: "text", log: "text",
};

/** The Material Symbols glyph drawn for each kind. */
export const KIND_ICON: Record<FileKind, string> = {
  image: "image",
  video: "movie",
  audio: "audio_file",
  pdf: "picture_as_pdf",
  text: "description",
  file: "draft",
  link: "link",
};

export function extensionOf(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export function fileKindFor(attachment: Attachment): FileKind {
  if (attachment.kind === "link") return "link";
  return KIND_BY_EXTENSION[extensionOf(attachment.name)] ?? "file";
}

/** The base name shown in the rename field, and the extension shown beside it (empty for links). */
export function splitName(attachment: Attachment): { base: string; tail: string } {
  if (attachment.kind === "link") return { base: attachment.name, tail: "" };
  const dot = attachment.name.lastIndexOf(".");
  if (dot <= 0) return { base: attachment.name, tail: "" };
  return { base: attachment.name.slice(0, dot), tail: attachment.name.slice(dot) };
}
