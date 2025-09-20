// SPDX-License-Identifier: Apache-2.0

/**
 * Service Worker for Hakanai PWA
 * Handles Share Target API requests while maintaining zero-knowledge
 */

/// <reference lib="webworker" />
declare const self: ServiceWorkerGlobalScope & typeof globalThis;

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
    const registration = await navigator.serviceWorker.register("/service-worker.js", {
      scope: "/",
    });
    console.log("Service Worker registered successfully:", registration);
  } catch (error) {
    console.error("Service Worker registration failed:", error);
  }
}

/**
 * File data structure for shared files
 */
export interface ShareTargetFile {
  name: string;
  type: string;
  size: number;
  data: string; // base64 encoded
}

/**
 * Share target data structure
 */
export interface ShareTargetData {
  title: string;
  text: string;
  url: string;
  timestamp: number;
  file?: ShareTargetFile;
}

// Install event - claim clients immediately
self.addEventListener("install", (event: ExtendableEvent) => {
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener("activate", (event: ExtendableEvent) => {
  event.waitUntil(self.clients.claim());
});

// Intercept specific fetch requests
self.addEventListener("fetch", (event: FetchEvent) => {
  const url = new URL(event.request.url);

  // Only intercept POST requests to /share from Share Target
  if (url.pathname === "/share" && event.request.method === "POST") {
    event.respondWith(handleShareTarget(event.request));
  }
});

/**
 * Convert ArrayBuffer to base64 string
 */
function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  const chunkSize = 8192; // Process in 8KB chunks to avoid stack overflow
  let binary = "";

  for (let i = 0; i < bytes.length; i += chunkSize) {
    const chunk = bytes.slice(i, i + chunkSize);
    binary += String.fromCharCode(...chunk);
  }

  return btoa(binary);
}

/**
 * Extract text data from FormData
 */
function extractTextData(formData: FormData): ShareTargetData {
  return {
    title: (formData.get("title") as string) || "",
    text: (formData.get("text") as string) || "",
    url: (formData.get("url") as string) || "",
    timestamp: Date.now(),
  };
}

/**
 * Process a single file
 */
async function processFile(file: File): Promise<ShareTargetFile | null> {
  if (!file || file.size === 0) {
    return null;
  }

  const arrayBuffer = await file.arrayBuffer();
  const base64 = arrayBufferToBase64(arrayBuffer);

  return {
    name: file.name,
    type: file.type,
    size: file.size,
    data: base64,
  };
}

/**
 * Extract and process files from FormData
 */
async function extractFileData(formData: FormData): Promise<ShareTargetFile | null> {
  const files = formData.getAll("files") as File[];
  if (!files || files.length === 0) {
    return null;
  }

  // For now, handle the first file
  return processFile(files[0]);
}

/**
 * Store data in cache
 */
async function storeInCache(data: ShareTargetData): Promise<void> {
  const cache = await caches.open(SHARE_CACHE_NAME);
  await cache.put(
    SHARE_DATA_KEY,
    new Response(JSON.stringify(data), {
      headers: {
        "Content-Type": "application/json",
      },
    }),
  );
}

/**
 * Handle Share Target POST request
 * Gets data and stores it in cache, then redirects to /share
 */
async function handleShareTarget(request: Request): Promise<Response> {
  try {
    const formData = await request.formData();

    const shareData = extractTextData(formData);
    const fileData = await extractFileData(formData);
    if (fileData) {
      shareData.file = fileData;
    }

    await storeInCache(shareData);

    return Response.redirect("/share", 303);
  } catch (error) {
    console.error("Error handling share target:", error);
    // On error, redirect to share page anyway
    return Response.redirect("/share", 303);
  }
}

/**
 * Clean up old share data (called from client)
 */
self.addEventListener("message", (event: ExtendableMessageEvent) => {
  if (event.data && event.data.type === "CLEAR_SHARE_CACHE") {
    clearShareCache();
  }
});

async function clearShareCache(): Promise<void> {
  try {
    const cache = await caches.open(SHARE_CACHE_NAME);
    await cache.delete(SHARE_DATA_KEY);
  } catch (error) {
    console.error("Error clearing share cache:", error);
  }
}
