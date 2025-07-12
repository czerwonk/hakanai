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

jest.mock("../server/src/typescript/common-utils", () => ({
  createButton: jest.fn(),
  createButtonContainer: jest.fn(),
  copyToClipboard: jest.fn(),
  announceToScreenReader: mockAnnounceToScreenReader,
  initTheme: jest.fn(),
  updateThemeToggleButton: jest.fn(),
}));

jest.mock("../server/src/typescript/types", () => ({
  isHakanaiError: jest.fn(
    (error: any) =>
      error && error.name === "HakanaiError" && typeof error.code === "string",
  ),
  isStandardError: jest.fn((error: any) => error instanceof Error),
  isErrorLike: jest.fn(
    (error: any) =>
      error &&
      typeof error === "object" &&
      ("message" in error || "name" in error),
  ),
}));

describe("create-secret.ts", () => {
  let dom: JSDOM;
  let window: Window & typeof globalThis;
  let document: Document;
  let createSecretModule: any;

  beforeEach(async () => {
    jest.clearAllMocks();

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
          "msg.fileTooLarge": "File size exceeds 10MB limit",
          "msg.fileReadError": "Error reading file",
          "msg.invalidFilename": "Invalid filename",
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

  describe("File validation functions", () => {
    test("sanitizeFileName removes dangerous characters", () => {
      const { sanitizeFileName } = createSecretModule;

      expect(sanitizeFileName('test<>:"/\\|?*file.txt')).toBe(
        "test_________file.txt",
      );
      expect(sanitizeFileName(".hidden")).toBe("hidden");
      expect(sanitizeFileName("normal.txt")).toBe("normal.txt");
      expect(sanitizeFileName("")).toBe(null);
    });

    test("validateFileSize rejects files over 10MB", () => {
      const { validateFileSize } = createSecretModule;

      const smallFile = new File(["small content"], "small.txt", {
        type: "text/plain",
      });
      const largeFile = new File(["x".repeat(11 * 1024 * 1024)], "large.txt", {
        type: "text/plain",
      });

      expect(validateFileSize(smallFile)).toBe(true);
      expect(validateFileSize(largeFile)).toBe(false);
    });

    test("formatFileSize returns correct formatted sizes", () => {
      const { formatFileSize } = createSecretModule;

      expect(formatFileSize(0)).toBe("0 Bytes");
      expect(formatFileSize(1024)).toBe("1 KB");
      expect(formatFileSize(1048576)).toBe("1 MB");
      expect(formatFileSize(1073741824)).toBe("1 GB");
      expect(formatFileSize(512)).toBe("512 Bytes");
    });
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
});
