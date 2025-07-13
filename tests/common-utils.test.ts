import {
  saveAuthTokenToStorage,
  getAuthTokenFromStorage,
  clearAuthTokenStorage,
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
