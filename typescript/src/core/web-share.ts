// SPDX-License-Identifier: Apache-2.0

import { I18nKeys } from "./i18n.js";

/**
 * Web Share API functionality for native sharing
 */

export interface ShareOptions {
  title?: string;
  text?: string;
  url?: string;
  files?: File[];
}

/**
 * Common MIME type mappings for file extensions
 */
const MIME_MAP: Record<string, string> = {
  // Images
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  png: "image/png",
  gif: "image/gif",
  webp: "image/webp",
  heic: "image/heic",
  heif: "image/heif",
  svg: "image/svg+xml",
  bmp: "image/bmp",
  ico: "image/x-icon",

  // Documents
  pdf: "application/pdf",
  doc: "application/msword",
  docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
  xls: "application/vnd.ms-excel",
  xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  ppt: "application/vnd.ms-powerpoint",
  pptx: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
  odt: "application/vnd.oasis.opendocument.text",

  // Videos
  mp4: "video/mp4",
  mov: "video/quicktime",
  avi: "video/x-msvideo",
  webm: "video/webm",
  mkv: "video/x-matroska",
  m4v: "video/x-m4v",

  // Audio
  mp3: "audio/mpeg",
  wav: "audio/wav",
  m4a: "audio/mp4",
  ogg: "audio/ogg",
  flac: "audio/flac",

  // Text/Code
  txt: "text/plain",
  json: "application/json",
  xml: "application/xml",
  html: "text/html",
  css: "text/css",
  js: "text/javascript",
  ts: "text/typescript",
  md: "text/markdown",
  csv: "text/csv",

  // Archives
  zip: "application/zip",
  tar: "application/x-tar",
  gz: "application/gzip",
  "7z": "application/x-7z-compressed",
  rar: "application/vnd.rar",
};

/**
 * Get MIME type from filename
 * @param filename The filename to extract extension from (always provided due to generateFilename)
 * @returns MIME type string
 */
export function getMimeType(filename: string): string {
  const ext = filename.split(".").pop()?.toLowerCase();
  return ext && MIME_MAP[ext] ? MIME_MAP[ext] : "application/octet-stream";
}

/**
 * Check if Web Share API is available
 */
export function isWebShareSupported(): boolean {
  return typeof navigator !== "undefined" && "share" in navigator;
}

/**
 * Check if file sharing is supported
 */
export function isFileShareSupported(): boolean {
  return typeof navigator !== "undefined" && "canShare" in navigator;
}

/**
 * Create a File object from bytes and filename
 * @param bytes The file content as Uint8Array
 * @param filename The filename
 * @returns File object
 */
export function createShareableFile(bytes: ArrayBuffer, filename: string): File {
  const mimeType = getMimeType(filename);
  return new File([bytes], filename, { type: mimeType });
}

/**
 * Validate share options before sharing
 * @param options Share options to validate
 * @throws Error if validation fails
 */
function validateShareOptions(options: ShareOptions): void {
  if (!options.title && !options.text && !options.url && !options.files) {
    throw new Error(window.i18n.t(I18nKeys.Error.ShareOptionsMissing));
  }

  if (options.files && options.files.length > 0) {
    if (!isFileShareSupported()) {
      throw new Error(window.i18n.t(I18nKeys.Error.FileShareNotSupported));
    }

    if (!navigator.canShare({ files: options.files })) {
      throw new Error(window.i18n.t(I18nKeys.Error.CannotShareFiles));
    }
  }
}

/**
 * Share content using Web Share API
 * @param options Share options
 * @returns Promise that resolves when sharing is complete or rejects if cancelled/failed
 */
export async function shareContent(options: ShareOptions): Promise<void> {
  if (!isWebShareSupported()) {
    throw new Error(window.i18n.t(I18nKeys.Error.WebShareNotSupported));
  }

  validateShareOptions(options);

  try {
    await navigator.share(options);
  } catch (error) {
    if (error instanceof Error) {
      if (error.name === "AbortError") {
        return;
      }

      throw error;
    }

    throw new Error("Share failed");
  }
}
