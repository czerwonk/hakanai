import { initTheme } from "../../server/src/typescript/core/theme";

describe("Theme Management", () => {
  let mockLocalStorage: { [key: string]: string };

  beforeEach(() => {
    // Mock localStorage
    mockLocalStorage = {};
    Object.defineProperty(window, "localStorage", {
      value: {
        getItem: jest.fn((key: string) => mockLocalStorage[key] || null),
        setItem: jest.fn((key: string, value: string) => {
          mockLocalStorage[key] = value;
        }),
        removeItem: jest.fn((key: string) => {
          delete mockLocalStorage[key];
        }),
      },
      writable: true,
    });

    // Mock matchMedia
    Object.defineProperty(window, "matchMedia", {
      writable: true,
      value: jest.fn().mockImplementation((query) => ({
        matches: query.includes("dark"),
        media: query,
        onchange: null,
        addListener: jest.fn(),
        removeListener: jest.fn(),
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
        dispatchEvent: jest.fn(),
      })),
    });

    // Mock i18n
    Object.defineProperty(window, "i18n", {
      value: {
        t: jest.fn((key: string) => {
          const translations: Record<string, string> = {
            "aria.switchToLight": "Switch to light mode",
            "aria.switchToDark": "Switch to dark mode",
          };
          return translations[key] || key;
        }),
      },
      writable: true,
    });

    // Clear document theme attributes
    document.body.removeAttribute("data-theme");
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("initTheme", () => {
    test("should apply stored theme preference", () => {
      mockLocalStorage["hakanai-theme"] = "dark";

      initTheme();

      expect(document.body.getAttribute("data-theme")).toBe("dark");
    });

    test("should apply system preference when no stored preference", () => {
      // Mock system preference for dark mode
      (window.matchMedia as jest.Mock).mockReturnValue({
        matches: true,
        addEventListener: jest.fn(),
      });

      initTheme();

      expect(document.body.getAttribute("data-theme")).toBe(null);
    });

    test("should apply light theme as fallback", () => {
      // Mock system preference for light mode
      (window.matchMedia as jest.Mock).mockReturnValue({
        matches: false,
        addEventListener: jest.fn(),
      });

      initTheme();

      expect(document.body.getAttribute("data-theme")).toBe(null);
    });

    test("should add event listener for system theme changes", () => {
      const mockAddEventListener = jest.fn();
      (window.matchMedia as jest.Mock).mockReturnValue({
        matches: false,
        addEventListener: mockAddEventListener,
      });

      initTheme();

      expect(mockAddEventListener).toHaveBeenCalledWith(
        "change",
        expect.any(Function),
      );
    });
  });
});
