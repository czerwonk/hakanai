// SPDX-License-Identifier: Apache-2.0

import { I18nKeys } from "./i18n";
import { getMimeType } from "./file-utils";

// Re-export getMimeType for backward compatibility
export { getMimeType };

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
