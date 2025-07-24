import { initFeatures } from "../../server/src/typescript/core/app-config";

describe("app-config", () => {
  let fetchMock: jest.Mock;
  let consoleWarnSpy: jest.SpyInstance;

  beforeEach(() => {
    // Setup DOM
    document.body.innerHTML = `
      <div id="impressum-link" class="hidden"></div>
      <div id="privacy-link" class="hidden"></div>
    `;

    // Mock fetch
    fetchMock = jest.fn();
    global.fetch = fetchMock as any;

    // Mock console.warn
    consoleWarnSpy = jest.spyOn(console, "warn").mockImplementation(() => {});
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  describe("initFeatures", () => {
    it("should show links when features are enabled in config", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          features: {
            impressum: true,
            privacy: true,
          },
        }),
      });

      await initFeatures();

      const impressumLink = document.getElementById("impressum-link");
      const privacyLink = document.getElementById("privacy-link");

      expect(impressumLink?.classList.contains("hidden")).toBe(false);
      expect(privacyLink?.classList.contains("hidden")).toBe(false);
    });

    it("should hide links when features are disabled in config", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          features: {
            impressum: false,
            privacy: false,
          },
        }),
      });

      await initFeatures();

      const impressumLink = document.getElementById("impressum-link");
      const privacyLink = document.getElementById("privacy-link");

      expect(impressumLink?.classList.contains("hidden")).toBe(true);
      expect(privacyLink?.classList.contains("hidden")).toBe(true);
    });

    it("should show only enabled features", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          features: {
            impressum: true,
            privacy: false,
          },
        }),
      });

      await initFeatures();

      const impressumLink = document.getElementById("impressum-link");
      const privacyLink = document.getElementById("privacy-link");

      expect(impressumLink?.classList.contains("hidden")).toBe(false);
      expect(privacyLink?.classList.contains("hidden")).toBe(true);
    });

    it("should hide all links when config fetch fails", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 404,
      });

      await initFeatures();

      const impressumLink = document.getElementById("impressum-link");
      const privacyLink = document.getElementById("privacy-link");

      expect(impressumLink?.classList.contains("hidden")).toBe(true);
      expect(privacyLink?.classList.contains("hidden")).toBe(true);
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        "Failed to fetch app config:",
        404,
      );
    });

    it("should hide all links when fetch throws an error", async () => {
      const error = new Error("Network error");
      fetchMock.mockRejectedValueOnce(error);

      await initFeatures();

      const impressumLink = document.getElementById("impressum-link");
      const privacyLink = document.getElementById("privacy-link");

      expect(impressumLink?.classList.contains("hidden")).toBe(true);
      expect(privacyLink?.classList.contains("hidden")).toBe(true);
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        "Failed to fetch app config:",
        error,
      );
    });

    it("should hide all links when config has missing features", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          // Missing features property
        }),
      });

      await initFeatures();

      const impressumLink = document.getElementById("impressum-link");
      const privacyLink = document.getElementById("privacy-link");

      expect(impressumLink?.classList.contains("hidden")).toBe(true);
      expect(privacyLink?.classList.contains("hidden")).toBe(true);
    });

    it("should handle missing DOM elements gracefully", async () => {
      // Remove one element
      document.getElementById("impressum-link")?.remove();

      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          features: {
            impressum: true,
            privacy: true,
          },
        }),
      });

      await initFeatures();

      // Should not throw error
      const privacyLink = document.getElementById("privacy-link");
      expect(privacyLink?.classList.contains("hidden")).toBe(false);
    });

    it("should handle elements that are already visible", async () => {
      // Remove hidden class from impressum link
      const impressumLink = document.getElementById("impressum-link");
      impressumLink?.classList.remove("hidden");

      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          features: {
            impressum: false,
            privacy: true,
          },
        }),
      });

      await initFeatures();

      // Impressum should be hidden again
      expect(impressumLink?.classList.contains("hidden")).toBe(true);

      const privacyLink = document.getElementById("privacy-link");
      expect(privacyLink?.classList.contains("hidden")).toBe(false);
    });

    it("should fetch config from correct endpoint", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          features: {
            impressum: true,
            privacy: true,
          },
        }),
      });

      await initFeatures();

      expect(fetchMock).toHaveBeenCalledWith("/config.json");
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });
  });
});

