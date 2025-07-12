import {
  createButton,
  createButtonContainer,
  copyToClipboard,
  announceToScreenReader,
  debounce,
  getTheme,
  applyTheme,
  toggleTheme,
  updateThemeToggleButton,
  initTheme,
} from "../server/src/typescript/common-utils";

// Mock localStorage
const mockLocalStorage = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};

// Mock matchMedia
const mockMatchMedia = jest.fn();

// Mock navigator.clipboard
const mockClipboard = {
  writeText: jest.fn(),
};

describe("Common Utils", () => {
  beforeEach(() => {
    // Reset DOM
    document.body.innerHTML = "";

    // Reset mocks
    jest.clearAllMocks();
    jest.useFakeTimers();

    // Suppress console.warn for tests
    jest.spyOn(console, "warn").mockImplementation();

    // Mock localStorage
    Object.defineProperty(window, "localStorage", {
      value: mockLocalStorage,
      writable: true,
    });

    // Mock matchMedia
    Object.defineProperty(window, "matchMedia", {
      value: mockMatchMedia,
      writable: true,
    });

    // Mock navigator.clipboard
    Object.defineProperty(navigator, "clipboard", {
      value: mockClipboard,
      writable: true,
    });

    // Default matchMedia mock
    mockMatchMedia.mockReturnValue({
      matches: false,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
    });
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe("DOM Creation Functions", () => {
    describe("createButton", () => {
      test("should create button with correct properties", () => {
        const clickHandler = jest.fn();
        const button = createButton(
          "test-class",
          "Test Text",
          "Test Label",
          clickHandler,
        );

        expect(button.tagName).toBe("BUTTON");
        expect(button.className).toBe("test-class");
        expect(button.type).toBe("button");
        expect(button.textContent).toBe("Test Text");
        expect(button.getAttribute("aria-label")).toBe("Test Label");
      });

      test("should attach click handler", () => {
        const clickHandler = jest.fn();
        const button = createButton(
          "test-class",
          "Test",
          "Label",
          clickHandler,
        );

        button.click();
        expect(clickHandler).toHaveBeenCalledTimes(1);
      });
    });

    describe("createButtonContainer", () => {
      test("should create div with correct class", () => {
        const container = createButtonContainer();

        expect(container.tagName).toBe("DIV");
        expect(container.className).toBe("buttons-container");
      });
    });
  });

  describe("Clipboard Operations", () => {
    describe("copyToClipboard", () => {
      test("should use clipboard API when available", async () => {
        mockClipboard.writeText.mockResolvedValue(undefined);

        const button = document.createElement("button");
        button.textContent = "Copy";

        copyToClipboard("test text", button, "Copy", "Copied!", "Failed");

        expect(mockClipboard.writeText).toHaveBeenCalledWith("test text");

        // Wait for promise resolution
        await Promise.resolve();

        expect(button.textContent).toBe("Copied!");
        expect(button.classList.contains("copied")).toBe(true);
      });

      test("should show visual feedback when clipboard API fails", async () => {
        jest.useRealTimers(); // Use real timers for this test

        mockClipboard.writeText.mockRejectedValue(
          new Error("Clipboard failed"),
        );

        // Mock document.execCommand to also fail (fallback)
        document.execCommand = jest.fn().mockReturnValue(false);

        const button = document.createElement("button");
        button.textContent = "Copy";

        copyToClipboard("test text", button, "Copy", "Copied!", "Failed");

        expect(mockClipboard.writeText).toHaveBeenCalledWith("test text");

        // Wait for promise rejection
        await new Promise((resolve) => setTimeout(resolve, 0));

        // Should show visual feedback instead of alert
        expect(button.textContent).toBe("Failed");
        expect(button.classList.contains("copy-failed")).toBe(true);

        jest.useFakeTimers(); // Restore fake timers
      });

      test("should use fallback when clipboard API is not available", () => {
        // Remove clipboard API
        Object.defineProperty(navigator, "clipboard", {
          value: undefined,
          writable: true,
        });

        // Mock document.execCommand to succeed (fallback method)
        document.execCommand = jest.fn().mockReturnValue(true);

        const button = document.createElement("button");
        button.textContent = "Copy";

        copyToClipboard("test text", button, "Copy", "Copied!", "Failed");

        // Should use fallback method successfully
        expect(document.execCommand).toHaveBeenCalledWith("copy");
        expect(button.textContent).toBe("Copied!");
        expect(button.classList.contains("copied")).toBe(true);
      });

      test("should show visual feedback when both clipboard API and fallback fail", () => {
        // Remove clipboard API
        Object.defineProperty(navigator, "clipboard", {
          value: undefined,
          writable: true,
        });

        // Mock document.execCommand to fail (fallback method)
        document.execCommand = jest.fn().mockReturnValue(false);

        const button = document.createElement("button");
        button.textContent = "Copy";

        copyToClipboard("test text", button, "Copy", "Copied!", "Failed");

        // Should show visual failure feedback
        expect(document.execCommand).toHaveBeenCalledWith("copy");
        expect(button.textContent).toBe("Failed");
        expect(button.classList.contains("copy-failed")).toBe(true);
      });

      test("should restore button state after timeout", async () => {
        mockClipboard.writeText.mockResolvedValue(undefined);

        const button = document.createElement("button");
        button.textContent = "Copy";

        copyToClipboard("test text", button, "Copy", "Copied!", "Failed");

        await Promise.resolve();

        expect(button.textContent).toBe("Copied!");

        // Fast forward timer
        jest.advanceTimersByTime(2000);

        expect(button.textContent).toBe("Copy");
        expect(button.classList.contains("copied")).toBe(false);
      });
    });
  });

  describe("Accessibility Functions", () => {
    describe("announceToScreenReader", () => {
      test("should create and append announcement element", () => {
        announceToScreenReader("Test message");

        const announcements = document.querySelectorAll('[role="status"]');
        expect(announcements).toHaveLength(1);

        const announcement = announcements[0] as HTMLElement;
        expect(announcement.getAttribute("aria-live")).toBe("polite");
        expect(announcement.className).toBe("sr-only");
        expect(announcement.textContent).toBe("Test message");
      });

      test("should remove announcement after timeout", () => {
        announceToScreenReader("Test message");

        expect(document.querySelectorAll('[role="status"]')).toHaveLength(1);

        jest.advanceTimersByTime(1000);

        expect(document.querySelectorAll('[role="status"]')).toHaveLength(0);
      });
    });
  });

  describe("Utility Functions", () => {
    describe("debounce", () => {
      test("should delay function execution", () => {
        const mockFn = jest.fn();
        const debouncedFn = debounce(mockFn, 100);

        debouncedFn("arg1", "arg2");
        expect(mockFn).not.toHaveBeenCalled();

        jest.advanceTimersByTime(100);
        expect(mockFn).toHaveBeenCalledWith("arg1", "arg2");
      });

      test("should cancel previous execution when called multiple times", () => {
        const mockFn = jest.fn();
        const debouncedFn = debounce(mockFn, 100);

        debouncedFn("first");
        debouncedFn("second");
        debouncedFn("third");

        jest.advanceTimersByTime(100);

        expect(mockFn).toHaveBeenCalledTimes(1);
        expect(mockFn).toHaveBeenCalledWith("third");
      });
    });
  });

  describe("Theme Management", () => {
    describe("getTheme", () => {
      test("should return saved theme from localStorage", () => {
        mockLocalStorage.getItem.mockReturnValue("dark");

        const theme = getTheme();
        expect(theme).toBe("dark");
        expect(mockLocalStorage.getItem).toHaveBeenCalledWith("hakanai-theme");
      });

      test("should return null for invalid saved theme", () => {
        mockLocalStorage.getItem.mockReturnValue("invalid-theme");

        const theme = getTheme();
        expect(theme).toBe(null);
      });

      test("should return null when localStorage fails", () => {
        mockLocalStorage.getItem.mockImplementation(() => {
          throw new Error("localStorage disabled");
        });

        const theme = getTheme();
        expect(theme).toBe(null);
      });
    });

    describe("applyTheme", () => {
      test("should set data-theme attribute for valid theme", () => {
        applyTheme("dark");
        expect(document.body.getAttribute("data-theme")).toBe("dark");

        applyTheme("light");
        expect(document.body.getAttribute("data-theme")).toBe("light");
      });

      test("should remove data-theme attribute for null theme", () => {
        document.body.setAttribute("data-theme", "dark");

        applyTheme(null);
        expect(document.body.hasAttribute("data-theme")).toBe(false);
      });
    });

    describe("toggleTheme", () => {
      test("should toggle from light to dark", () => {
        // Set initial light theme
        document.body.setAttribute("data-theme", "light");

        toggleTheme();

        expect(mockLocalStorage.setItem).toHaveBeenCalledWith(
          "hakanai-theme",
          "dark",
        );
        expect(document.body.getAttribute("data-theme")).toBe("dark");
      });

      test("should toggle from dark to light", () => {
        // Set initial dark theme
        document.body.setAttribute("data-theme", "dark");

        toggleTheme();

        expect(mockLocalStorage.setItem).toHaveBeenCalledWith(
          "hakanai-theme",
          "light",
        );
        expect(document.body.getAttribute("data-theme")).toBe("light");
      });

      test("should handle localStorage errors gracefully", () => {
        mockLocalStorage.setItem.mockImplementation(() => {
          throw new Error("localStorage full");
        });

        document.body.setAttribute("data-theme", "light");

        // Should not throw
        expect(() => toggleTheme()).not.toThrow();
        expect(document.body.getAttribute("data-theme")).toBe("dark");
      });
    });

    describe("updateThemeToggleButton", () => {
      test("should update button text and aria-label for dark theme", () => {
        const button = document.createElement("button");
        button.id = "theme-toggle";
        document.body.appendChild(button);
        document.body.setAttribute("data-theme", "dark");

        updateThemeToggleButton();

        expect(button.textContent).toBe("🌙");
        expect(button.getAttribute("aria-label")).toBe("Switch to light mode");
      });

      test("should update button text and aria-label for light theme", () => {
        const button = document.createElement("button");
        button.id = "theme-toggle";
        document.body.appendChild(button);
        document.body.setAttribute("data-theme", "light");

        updateThemeToggleButton();

        expect(button.textContent).toBe("☀️");
        expect(button.getAttribute("aria-label")).toBe("Switch to dark mode");
      });

      test("should handle missing button gracefully", () => {
        expect(() => updateThemeToggleButton()).not.toThrow();
      });

      test("should use i18n translations when available", () => {
        const button = document.createElement("button");
        button.id = "theme-toggle";
        document.body.appendChild(button);
        document.body.setAttribute("data-theme", "dark");

        // Mock i18n
        (window as any).i18n = {
          t: jest.fn().mockReturnValue("Zum hellen Modus wechseln"),
        };

        updateThemeToggleButton();

        expect(button.getAttribute("aria-label")).toBe(
          "Zum hellen Modus wechseln",
        );
      });
    });

    describe("initTheme", () => {
      test("should initialize theme from localStorage", () => {
        mockLocalStorage.getItem.mockReturnValue("dark");

        initTheme();

        expect(document.body.getAttribute("data-theme")).toBe("dark");
      });

      test("should create theme toggle button if it does not exist", () => {
        initTheme();

        const button = document.getElementById("theme-toggle");
        expect(button).toBeTruthy();
        expect(button?.tagName).toBe("BUTTON");
      });

      test("should insert button before language switcher if it exists", () => {
        const languageSwitcher = document.createElement("select");
        languageSwitcher.id = "language-switcher";
        const parent = document.createElement("div");
        parent.appendChild(languageSwitcher);
        document.body.appendChild(parent);

        initTheme();

        const button = document.getElementById("theme-toggle");
        expect(button?.nextElementSibling).toBe(languageSwitcher);
      });

      test("should setup system theme change listener", () => {
        const mockAddEventListener = jest.fn();
        mockMatchMedia.mockReturnValue({
          matches: false,
          addEventListener: mockAddEventListener,
          removeEventListener: jest.fn(),
        });

        initTheme();

        expect(mockMatchMedia).toHaveBeenCalledWith(
          "(prefers-color-scheme: dark)",
        );
        expect(mockAddEventListener).toHaveBeenCalledWith(
          "change",
          expect.any(Function),
        );
      });
    });
  });
});
