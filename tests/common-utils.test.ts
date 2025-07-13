import {
  saveAuthTokenToStorage,
  getAuthTokenFromStorage,
  clearAuthTokenStorage,
} from "../server/src/typescript/common-utils";

describe("LocalStorage Auth Token Management", () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
    jest.clearAllMocks();
  });

  describe("saveAuthTokenToStorage", () => {
    test("should save token to localStorage", () => {
      const result = saveAuthTokenToStorage("test-token-123");

      expect(result).toBe(true);

      const stored = localStorage.getItem("hakanai-auth-token");
      expect(stored).toBeTruthy();

      const tokenData = JSON.parse(stored!);
      expect(tokenData.token).toBe("test-token-123");
      expect(tokenData.expires).toBeGreaterThan(Date.now());
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
    test("should retrieve token from localStorage", () => {
      const tokenData = {
        token: "test-token-123",
        expires: Date.now() + 24 * 60 * 60 * 1000, // 24 hours from now
      };
      localStorage.setItem("hakanai-auth-token", JSON.stringify(tokenData));

      const result = getAuthTokenFromStorage();
      expect(result).toBe("test-token-123");
    });

    test("should return null when token not found", () => {
      const result = getAuthTokenFromStorage();
      expect(result).toBe(null);
    });

    test("should return null and clean up expired token", () => {
      const tokenData = {
        token: "expired-token",
        expires: Date.now() - 1000, // 1 second ago (expired)
      };
      localStorage.setItem("hakanai-auth-token", JSON.stringify(tokenData));

      const result = getAuthTokenFromStorage();
      expect(result).toBe(null);
      expect(localStorage.getItem("hakanai-auth-token")).toBe(null);
    });

    test("should handle corrupted JSON data", () => {
      localStorage.setItem("hakanai-auth-token", "invalid-json-data");

      const result = getAuthTokenFromStorage();
      expect(result).toBe(null);
      expect(localStorage.getItem("hakanai-auth-token")).toBe(null);
    });
  });

  describe("clearAuthTokenStorage", () => {
    test("should clear token from localStorage", () => {
      const tokenData = {
        token: "test-token-123",
        expires: Date.now() + 24 * 60 * 60 * 1000,
      };
      localStorage.setItem("hakanai-auth-token", JSON.stringify(tokenData));

      clearAuthTokenStorage();

      expect(localStorage.getItem("hakanai-auth-token")).toBe(null);
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

