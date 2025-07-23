interface AppConfig {
  features: {
    impressum: boolean;
    privacy: boolean;
  };
}

/**
 * Fetch application configuration from server
 */
async function fetchAppConfig(): Promise<AppConfig | null> {
  try {
    const response = await fetch("/config.json");
    if (!response.ok) {
      console.warn("Failed to fetch app config:", response.status);
      return null;
    }
    return await response.json();
  } catch (error) {
    console.warn("Failed to fetch app config:", error);
    return null;
  }
}

/**
 * Initialize UI based on application configuration
 */
async function initFeatures(): Promise<void> {
  const config = await fetchAppConfig();
  await initializeOptionalFeature(
    "impressum-link",
    config?.features.impressum ?? false,
  );
  await initializeOptionalFeature(
    "privacy-link",
    config?.features.privacy ?? false,
  );
}

async function initializeOptionalFeature(
  elementId: string,
  enabled: boolean,
): Promise<void> {
  const element = document.getElementById(elementId);
  if (element) {
    if (enabled) {
      element?.classList.remove("hidden");
    } else {
      element?.classList.add("hidden");
    }
  }
}

export { initFeatures };
