import {
  saveAuthTokenToStorage,
  getAuthTokenFromStorage,
  clearAuthTokenStorage,
  formatFileSize,
} from "../server/src/typescript/common-utils";

describe("SessionStorage Auth Token Management", () => {
  beforeEach(() => {
    // Clear sessionStorage before each test
    sessionStorage.clear();
    jest.clearAllMocks();
  });

  describe("saveAuthTokenToStorage", () => {
    test("should save token to sessionStorage", () => {
      const result = saveAuthTokenToStorage("test-token-123");

      expect(result).toBe(true);

      const stored = sessionStorage.getItem("hakanai-auth-token");
      expect(stored).toBe("test-token-123");
    });

    test("should return false for empty token", () => {
      const result = saveAuthTokenToStorage("");
      expect(result).toBe(false);
    });

    test("should return false for whitespace token", () => {
      const result = saveAuthTokenToStorage("   ");
      expect(result).toBe(false);
    });
  });

  describe("getAuthTokenFromStorage", () => {
    test("should retrieve token from sessionStorage", () => {
      sessionStorage.setItem("hakanai-auth-token", "test-token-123");

      const result = getAuthTokenFromStorage();
      expect(result).toBe("test-token-123");
    });

    test("should return null when token not found", () => {
      const result = getAuthTokenFromStorage();
      expect(result).toBe(null);
    });
  });

  describe("clearAuthTokenStorage", () => {
    test("should clear token from sessionStorage", () => {
      sessionStorage.setItem("hakanai-auth-token", "test-token-123");

      clearAuthTokenStorage();

      expect(sessionStorage.getItem("hakanai-auth-token")).toBe(null);
    });
  });

  describe("Storage roundtrip", () => {
    test("should save and retrieve token correctly", () => {
      const saveResult = saveAuthTokenToStorage("my-secret-token");
      expect(saveResult).toBe(true);

      const retrievedToken = getAuthTokenFromStorage();
      expect(retrievedToken).toBe("my-secret-token");
    });
  });
});

describe("formatFileSize", () => {
  test("returns correct formatted sizes", () => {
    expect(formatFileSize(0)).toBe("0 Bytes");
    expect(formatFileSize(1024)).toBe("1 KB");
    expect(formatFileSize(1048576)).toBe("1 MB");
    expect(formatFileSize(1073741824)).toBe("1 GB");
    expect(formatFileSize(512)).toBe("512 Bytes");
  });

  test("handles decimal values correctly", () => {
    expect(formatFileSize(1536)).toBe("1.5 KB");
    expect(formatFileSize(1572864)).toBe("1.5 MB");
    expect(formatFileSize(2048)).toBe("2 KB");
  });

  test("handles large files", () => {
    expect(formatFileSize(5368709120)).toBe("5 GB");
    expect(formatFileSize(10737418240)).toBe("10 GB");
  });
});
