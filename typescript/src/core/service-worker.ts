// SPDX-License-Identifier: Apache-2.0

/**
 * Service Worker for Hakanai PWA
 * Handles Share Target API requests while maintaining zero-knowledge
 */

export const SHARE_CACHE_NAME = "hakanai-share-v1";
export const SHARE_DATA_KEY = "/share-target-data";

/**
 * Register service worker for PWA functionality
 */
export async function registerServiceWorker() {
  if (!("serviceWorker" in navigator)) {
    return;
  }

  try {
    const registration = await navigator.serviceWorker.register("/sw.js", {
      scope: "/share",
    });
    console.log("Service Worker registered successfully:", registration);
  } catch (error) {
    console.error("Service Worker registration failed:", error);
  }
}
