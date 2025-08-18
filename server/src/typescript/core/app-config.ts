// SPDX-License-Identifier: Apache-2.0

import { showElement, hideElement } from "./dom-utils";

export interface AppConfig {
  features: {
    impressum: boolean;
    privacy: boolean;
    showTokenInput: boolean;
  };
}

let cachedConfig: AppConfig | null | undefined;

/**
 * Fetch application configuration from server (with caching)
 */
export async function fetchAppConfig(): Promise<AppConfig | null> {
  if (cachedConfig !== undefined) {
    return cachedConfig;
  }

  try {
    const response = await fetch("/config.json");
    if (!response.ok) {
      console.warn("Failed to fetch app config:", response.status);
      cachedConfig = null;
      return null;
    }
    const config: AppConfig = await response.json();
    cachedConfig = config;

    return config;
  } catch (error) {
    console.warn("Failed to fetch app config:", error);
    cachedConfig = null;
    return null;
  }
}

async function initializeOptionalFeature(
  elementId: string,
  enabled: boolean,
): Promise<void> {
  const element = document.getElementById(elementId);
  if (!element) {
    return;
  }

  if (enabled) {
    showElement(element);
  } else {
    hideElement(element);
  }
}

/**
 * Initialize UI based on application configuration
 */
export async function initFeatures(): Promise<void> {
  const config = await fetchAppConfig();
  if (!config) {
    console.warn("No configuration found, skipping feature initialization.");
    return;
  }

  await initializeOptionalFeature(
    "impressum-link",
    config.features?.impressum ?? false,
  );
  await initializeOptionalFeature(
    "privacy-link",
    config.features?.privacy ?? false,
  );
}
