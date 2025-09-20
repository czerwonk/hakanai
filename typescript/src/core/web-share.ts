// SPDX-License-Identifier: Apache-2.0

/**
 * Web Share API functionality for native sharing
 */

export interface ShareOptions {
  title?: string;
  text?: string;
  url?: string;
}

/**
 * Check if Web Share API is available
 */
export function isWebShareSupported(): boolean {
  return typeof navigator !== "undefined" && "share" in navigator;
}

/**
 * Share content using Web Share API
 * @param options Share options
 * @returns Promise that resolves when sharing is complete or rejects if cancelled/failed
 */
export async function shareContent(options: ShareOptions): Promise<void> {
  if (!isWebShareSupported()) {
    throw new Error("Web Share API is not supported in this browser");
  }

  // Validate that at least one field is provided
  if (!options.title && !options.text && !options.url) {
    throw new Error("At least one of title, text, or url must be provided");
  }

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
