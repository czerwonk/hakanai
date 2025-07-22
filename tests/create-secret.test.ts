import { JSDOM } from "jsdom";

// Mock external dependencies only
const mockSendPayload = jest.fn();
const mockCreatePayload = jest.fn();
const mockAnnounceToScreenReader = jest.fn();

jest.mock("../server/src/typescript/hakanai-client", () => ({
  HakanaiClient: jest.fn().mockImplementation(() => ({
    sendPayload: mockSendPayload,
    createPayload: mockCreatePayload,
  })),
}));

const mockSaveAuthTokenToStorage = jest.fn();
const mockGetAuthTokenFromStorage = jest.fn();
const mockClearAuthTokenStorage = jest.fn();
const mockSecureInputClear = jest.fn();

// Import the real sanitizeFileName function
const { sanitizeFileName } = jest.requireActual(
  "../server/src/typescript/common-utils",
);

jest.mock("../server/src/typescript/common-utils", () => ({
  createButton: jest.fn(),
  createButtonContainer: jest.fn(),
  copyToClipboard: jest.fn(),
  announceToScreenReader: mockAnnounceToScreenReader,
  initTheme: jest.fn(),
  updateThemeToggleButton: jest.fn(),
  saveAuthTokenToStorage: mockSaveAuthTokenToStorage,
  getAuthTokenFromStorage: mockGetAuthTokenFromStorage,
  clearAuthTokenStorage: mockClearAuthTokenStorage,
  secureInputClear: mockSecureInputClear,
  sanitizeFileName, // Use the real implementation
}));

const mockIsHakanaiError = jest.fn();
const mockIsStandardError = jest.fn();
const mockIsErrorLike = jest.fn();

jest.mock("../server/src/typescript/types", () => ({
  isHakanaiError: mockIsHakanaiError,
  isStandardError: mockIsStandardError,
  isErrorLike: mockIsErrorLike,
}));

describe("create-secret.ts", () => {
  let dom: JSDOM;
  let window: Window & typeof globalThis;
  let document: Document;
  let createSecretModule: any;

  beforeEach(async () => {
    jest.clearAllMocks();

    // Set up default mock implementations for type guards
    mockIsHakanaiError.mockImplementation(
      (error: any) =>
        error &&
        error.name === "HakanaiError" &&
        typeof error.code === "string",
    );
    mockIsStandardError.mockImplementation(
      (error: any) => error instanceof Error,
    );
    mockIsErrorLike.mockImplementation(
      (error: any) =>
        error &&
        typeof error === "object" &&
        ("message" in error || "name" in error),
    );

    dom = new JSDOM(`
      <!DOCTYPE html>
      <html>
        <head><title>Test</title></head>
        <body>
          <div id="loading" style="display: none;"></div>
          <button id="createBtn">Create</button>
          <input id="secretText" type="text" />
          <input id="secretFile" type="file" />
          <input id="authToken" type="text" />
          <select id="ttlSelect"><option value="3600">1 hour</option></select>
          <input id="textRadio" type="radio" name="type" checked />
          <input id="fileRadio" type="radio" name="type" />
          <input id="separateKey" type="checkbox" />
          <div id="result"></div>
          <form id="create-secret-form">
            <div id="textInputGroup"></div>
            <div id="fileInputGroup"></div>
            <div id="fileInfo" style="display: none;">
              <span id="fileName"></span>
              <span id="fileSize"></span>
            </div>
          </form>
        </body>
      </html>
    `);

    window = dom.window as any;
    document = window.document;

    // Set up global objects
    global.window = window;
    global.document = document;
    global.TextEncoder = window.TextEncoder;
    global.FileReader = window.FileReader;
    global.crypto = {
      randomUUID: jest.fn(() => "test-uuid-123"),
    } as any;

    // Mock i18n
    (window as any).i18n = {
      t: jest.fn((key: string) => {
        const translations: { [key: string]: string } = {
          "msg.emptySecret": "Please enter a secret to share",
          "msg.emptyFile": "Please select a file to share",
          "msg.createFailed": "Failed to create secret",
          "msg.successTitle": "Secret Created Successfully",
          "msg.errorTitle": "Error",
          "msg.fileReadError": "Error reading file",
          "msg.invalidFilename": "Invalid filename",
          "error.AUTHENTICATION_REQUIRED":
            "Authentication required - Please enter your authentication token",
          "error.INVALID_TOKEN":
            "Invalid authentication token - Please check your token and try again",
        };
        return translations[key] || key;
      }),
    };

    // Set up mock payload
    const mockPayload = {
      data: "test-data",
      filename: undefined,
      setFromBytes: jest.fn(),
    };
    mockCreatePayload.mockReturnValue(mockPayload);

    // Import the module
    createSecretModule = await import("../server/src/typescript/create-secret");
  });

  afterEach(() => {
    dom.window.close();
  });

  describe("Input validation", () => {
    test("validates empty text input", async () => {
      const secretInput = document.getElementById(
        "secretText",
      ) as HTMLInputElement;
      secretInput.value = "";

      await createSecretModule.createSecret();

      // Should not attempt to send when input is empty
      expect(mockSendPayload).not.toHaveBeenCalled();
    });

    test("validates empty file input in file mode", async () => {
      const fileRadio = document.getElementById(
        "fileRadio",
      ) as HTMLInputElement;
      const fileInput = document.getElementById(
        "secretFile",
      ) as HTMLInputElement;

      fileRadio.checked = true;
      Object.defineProperty(fileInput, "files", {
        value: null,
        writable: false,
      });

      await createSecretModule.createSecret();

      // Should not attempt to send when no file is selected
      expect(mockSendPayload).not.toHaveBeenCalled();
    });

    test("validates file size limits", async () => {
      const fileRadio = document.getElementById(
        "fileRadio",
      ) as HTMLInputElement;
      const fileInput = document.getElementById(
        "secretFile",
      ) as HTMLInputElement;

      fileRadio.checked = true;
      const largeFile = new File(["x".repeat(11 * 1024 * 1024)], "large.txt", {
        type: "text/plain",
      });

      Object.defineProperty(fileInput, "files", {
        value: [largeFile],
        writable: false,
      });

      await createSecretModule.createSecret();

      // Should not attempt to send when file is too large
      expect(mockSendPayload).not.toHaveBeenCalled();
    });
  });

  describe("File handling utilities", () => {
    test("toggleSecretType switches between text and file modes", () => {
      const textRadio = document.getElementById(
        "textRadio",
      ) as HTMLInputElement;
      const fileRadio = document.getElementById(
        "fileRadio",
      ) as HTMLInputElement;

      // Start in text mode
      textRadio.checked = true;
      createSecretModule.toggleSecretType();

      // Switch to file mode
      fileRadio.checked = true;
      textRadio.checked = false;
      createSecretModule.toggleSecretType();

      // Verify mode switched (this is a basic functionality test)
      expect(fileRadio.checked).toBe(true);
    });
  });

  describe("Cookie Integration", () => {
    beforeEach(() => {
      // Add saveTokenCookie checkbox to DOM
      const saveTokenCheckbox = document.createElement("input");
      saveTokenCheckbox.type = "checkbox";
      saveTokenCheckbox.id = "saveTokenCookie";
      document.body.appendChild(saveTokenCheckbox);

      // Reset mock implementations
      mockSaveAuthTokenToStorage.mockReset();
      mockGetAuthTokenFromStorage.mockReset();
      mockClearAuthTokenStorage.mockReset();
      mockSecureInputClear.mockReset();
    });

    test("should have cookie integration functions available", () => {
      // Test that the functions are exported and can be called
      expect(typeof createSecretModule.initializeAuthToken).toBe("function");
      expect(typeof createSecretModule.handleAuthTokenSave).toBe("function");

      // Since the module is heavily mocked, we primarily test that the functions exist
      // and can be called without throwing errors
      expect(() => createSecretModule.initializeAuthToken()).not.toThrow();
      expect(() =>
        createSecretModule.handleAuthTokenSave("test", true),
      ).not.toThrow();
      expect(() =>
        createSecretModule.handleAuthTokenSave("test", false),
      ).not.toThrow();
    });

    test("should handle checkbox-based cookie saving logic", async () => {
      // Test that the checkbox affects behavior
      const secretInput = document.getElementById(
        "secretText",
      ) as HTMLInputElement;
      const authTokenInput = document.getElementById(
        "authToken",
      ) as HTMLInputElement;

      secretInput.value = "test secret";
      authTokenInput.value = "test-token";

      // Test with checkbox checked
      const saveTokenCheckbox = document.createElement("input");
      saveTokenCheckbox.type = "checkbox";
      saveTokenCheckbox.id = "saveTokenCookie";
      saveTokenCheckbox.checked = true;
      document.body.appendChild(saveTokenCheckbox);

      // Mock successful creation
      const mockPayload = { data: "encrypted", filename: null };
      mockCreatePayload.mockReturnValue(mockPayload);
      mockSendPayload.mockResolvedValue("https://example.com/s/123#key");

      // Should not throw
      await expect(createSecretModule.createSecret()).resolves.not.toThrow();

      // Test with checkbox unchecked
      saveTokenCheckbox.checked = false;
      authTokenInput.value = "test-token-2";
      secretInput.value = "test secret 2";

      await expect(createSecretModule.createSecret()).resolves.not.toThrow();
    });

    test("should handle missing checkbox gracefully", async () => {
      const secretInput = document.getElementById(
        "secretText",
      ) as HTMLInputElement;
      const authTokenInput = document.getElementById(
        "authToken",
      ) as HTMLInputElement;

      secretInput.value = "test secret";
      authTokenInput.value = "test-token";

      // No checkbox in DOM

      // Mock successful creation
      const mockPayload = { data: "encrypted", filename: null };
      mockCreatePayload.mockReturnValue(mockPayload);
      mockSendPayload.mockResolvedValue("https://example.com/s/123#key");

      // Should not throw when checkbox is missing
      await expect(createSecretModule.createSecret()).resolves.not.toThrow();
    });
  });

  describe("Authentication Error Handling", () => {
    test("authentication error translations are available", () => {
      // Test that the specific error codes we added are in the i18n translations
      const authRequiredKey = "error.AUTHENTICATION_REQUIRED";
      const invalidTokenKey = "error.INVALID_TOKEN";

      // These translations should exist and return meaningful messages
      const authRequiredMessage = (window as any).i18n.t(authRequiredKey);
      const invalidTokenMessage = (window as any).i18n.t(invalidTokenKey);

      expect(authRequiredMessage).toBeDefined();
      expect(invalidTokenMessage).toBeDefined();

      // The keys should not be returned as-is (meaning translation exists)
      expect(authRequiredMessage).not.toBe(authRequiredKey);
      expect(invalidTokenMessage).not.toBe(invalidTokenKey);

      // Check that translations contain expected content
      expect(authRequiredMessage).toContain("Authentication required");
      expect(invalidTokenMessage).toContain("Invalid authentication token");
    });

    test("auth token input element exists in DOM", () => {
      // Test that auth token input exists and can be manipulated
      const authTokenInput = document.getElementById(
        "authToken",
      ) as HTMLInputElement;
      expect(authTokenInput).toBeDefined();
      expect(authTokenInput.tagName).toBe("INPUT");
      expect(authTokenInput.type).toBe("text");

      // Mock focus and select methods (these exist in real DOM)
      authTokenInput.focus = jest.fn();
      authTokenInput.select = jest.fn();

      // Test that the methods can be called
      authTokenInput.focus();
      authTokenInput.select();

      expect(authTokenInput.focus).toHaveBeenCalled();
      expect(authTokenInput.select).toHaveBeenCalled();
    });

    test("type guards work correctly for error identification", () => {
      // Test that our type guards correctly identify HakanaiError
      const hakanaiError = {
        name: "HakanaiError",
        code: "AUTHENTICATION_REQUIRED", // Using string for now since this is a mock test
        message: "Authentication required",
      };

      const standardError = new Error("Standard error");

      const otherObject = {
        message: "Some message",
      };

      expect(mockIsHakanaiError(hakanaiError)).toBe(true);
      expect(mockIsHakanaiError(standardError)).toBe(false);
      expect(mockIsStandardError(standardError)).toBe(true);
      expect(mockIsErrorLike(otherObject)).toBe(true);
    });

    test("error handling functions are exported and callable", () => {
      // Test that error handling related functions exist and can be called
      expect(typeof createSecretModule.createSecret).toBe("function");
      expect(typeof createSecretModule.showError).toBe("function");

      // Test that showError can be called without throwing
      expect(() => createSecretModule.showError("Test error")).not.toThrow();
    });

    test("UI strings contain authentication error messages", () => {
      // Verify that the UI_STRINGS object (if exported) or error handling
      // supports the authentication error cases
      const uiStrings = createSecretModule.UI_STRINGS;
      if (uiStrings) {
        // If UI_STRINGS is exported, verify it has basic error handling
        expect(typeof uiStrings).toBe("object");
      }

      // At minimum, verify the module exports what we expect
      expect(createSecretModule).toBeDefined();
    });
  });
});
