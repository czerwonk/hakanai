import { initTheme } from "../../server/src/typescript/core/theme";

describe("Theme Management", () => {
  beforeEach(() => {
    // Clear document theme attributes
    document.body.removeAttribute("data-theme");

    // Mock matchMedia (always needed)
    Object.defineProperty(window, "matchMedia", {
      value: jest.fn(() => ({
        matches: false,
        addEventListener: jest.fn(),
      })),
      writable: true,
    });

    // Reset mocks
    jest.clearAllMocks();
  });

  test("applies stored dark theme preference", () => {
    // Mock localStorage with stored dark theme
    Object.defineProperty(window, "localStorage", {
      value: { getItem: () => "dark" },
      writable: true,
    });

    initTheme();

    expect(document.body.getAttribute("data-theme")).toBe("dark");
  });

  test("applies stored light theme preference", () => {
    // Mock localStorage with stored light theme
    Object.defineProperty(window, "localStorage", {
      value: { getItem: () => "light" },
      writable: true,
    });

    initTheme();

    expect(document.body.getAttribute("data-theme")).toBe("light");
  });

  test("uses system preference when no stored preference", () => {
    // Mock localStorage with no stored theme
    Object.defineProperty(window, "localStorage", {
      value: { getItem: () => null },
      writable: true,
    });

    initTheme();

    // Should not set explicit theme (relies on system)
    expect(document.body.getAttribute("data-theme")).toBe(null);
  });
});
