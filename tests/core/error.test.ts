import {
  handleAPIError,
  ErrorHandler,
  isHakanaiError,
  isStandardError,
  isErrorLike,
} from "../../server/src/typescript/core/error";
import { HakanaiErrorCodes } from "../../server/src/typescript/hakanai-client";

describe("Error Module", () => {
  beforeEach(() => {
    Object.defineProperty(window, "i18n", {
      value: {
        t: jest.fn((key: string) => {
          // Simulate translation - return translated text for error codes
          if (key.startsWith("error.")) {
            const code = key.replace("error.", "");
            return `Translated: ${code}`;
          }
          return key;
        }),
      },
      writable: true,
    });
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Type Guards", () => {
    describe("isHakanaiError", () => {
      test("should identify valid HakanaiError", () => {
        const hakanaiError = {
          name: "HakanaiError",
          code: "SEND_FAILED",
          message: "Failed to send",
        };

        expect(isHakanaiError(hakanaiError)).toBe(true);
      });

      test("should reject invalid objects", () => {
        expect(isHakanaiError(null)).toBe(false);
        expect(isHakanaiError(undefined)).toBe(false);
        expect(isHakanaiError("string")).toBe(false);
        expect(isHakanaiError({})).toBe(false);
        expect(isHakanaiError({ name: "Error" })).toBe(false);
        expect(isHakanaiError({ name: "HakanaiError" })).toBe(false);
        expect(isHakanaiError({ name: "HakanaiError", code: 123 })).toBe(false);
      });
    });

    describe("isStandardError", () => {
      test("should identify Error instances", () => {
        expect(isStandardError(new Error("test"))).toBe(true);
        expect(isStandardError(new TypeError("test"))).toBe(true);
      });

      test("should reject non-Error objects", () => {
        expect(isStandardError(null)).toBe(false);
        expect(isStandardError("error")).toBe(false);
        expect(isStandardError({ message: "error" })).toBe(false);
      });
    });

    describe("isErrorLike", () => {
      test("should identify error-like objects", () => {
        expect(isErrorLike({ message: "error" })).toBe(true);
        expect(isErrorLike({ name: "CustomError" })).toBe(true);
        expect(isErrorLike({ message: "error", name: "CustomError" })).toBe(
          true,
        );
      });

      test("should reject invalid objects", () => {
        expect(isErrorLike(null)).toBe(false);
        expect(isErrorLike(undefined)).toBe(false);
        expect(isErrorLike("string")).toBe(false);
        expect(isErrorLike({})).toBe(false);
        expect(isErrorLike(123)).toBe(false);
      });
    });
  });

  describe("handleAPIError", () => {
    let mockHandler: ErrorHandler;

    beforeEach(() => {
      mockHandler = {
        displayError: jest.fn(),
        onAuthenticationError: jest.fn(),
      };
    });

    test("should handle HakanaiError with translation", () => {
      const hakanaiError = {
        name: "HakanaiError",
        code: HakanaiErrorCodes.SEND_FAILED,
        message: "Original message",
      };

      handleAPIError(hakanaiError, "Fallback message", mockHandler);

      expect(mockHandler.displayError).toHaveBeenCalledWith(
        "Translated: SEND_FAILED",
      );
      expect(mockHandler.onAuthenticationError).not.toHaveBeenCalled();
    });

    test("should handle authentication errors specially", () => {
      const authError = {
        name: "HakanaiError",
        code: HakanaiErrorCodes.AUTHENTICATION_REQUIRED,
        message: "Auth required",
      };

      handleAPIError(authError, "Fallback", mockHandler);

      expect(mockHandler.displayError).toHaveBeenCalledWith(
        "Translated: AUTHENTICATION_REQUIRED",
      );
      expect(mockHandler.onAuthenticationError).toHaveBeenCalled();
    });

    test("should handle invalid token errors", () => {
      const tokenError = {
        name: "HakanaiError",
        code: HakanaiErrorCodes.INVALID_TOKEN,
        message: "Invalid token",
      };

      handleAPIError(tokenError, "Fallback", mockHandler);

      expect(mockHandler.displayError).toHaveBeenCalledWith(
        "Translated: INVALID_TOKEN",
      );
      expect(mockHandler.onAuthenticationError).toHaveBeenCalled();
    });

    test("should handle standard Error objects", () => {
      const standardError = new Error("Standard error message");

      handleAPIError(standardError, "Fallback", mockHandler);

      expect(mockHandler.displayError).toHaveBeenCalledWith(
        "Standard error message",
      );
      expect(mockHandler.onAuthenticationError).not.toHaveBeenCalled();
    });

    test("should handle error-like objects", () => {
      const errorLike = { message: "Error-like message" };

      handleAPIError(errorLike, "Fallback", mockHandler);

      expect(mockHandler.displayError).toHaveBeenCalledWith(
        "Error-like message",
      );
    });

    test("should handle error-like objects without message", () => {
      const errorLike = { name: "CustomError" };

      handleAPIError(errorLike, "Fallback message", mockHandler);

      expect(mockHandler.displayError).toHaveBeenCalledWith("Fallback message");
    });

    test("should handle unknown error types", () => {
      const unknownError = "string error";

      handleAPIError(unknownError, "Fallback message", mockHandler);

      expect(mockHandler.displayError).toHaveBeenCalledWith("Fallback message");
    });

    test("should handle null/undefined errors", () => {
      handleAPIError(null, "Fallback message", mockHandler);
      expect(mockHandler.displayError).toHaveBeenCalledWith("Fallback message");

      handleAPIError(undefined, "Another fallback", mockHandler);
      expect(mockHandler.displayError).toHaveBeenCalledWith("Another fallback");
    });

    test("should work with handler without optional methods", () => {
      const basicHandler: ErrorHandler = {
        displayError: jest.fn(),
      };

      const authError = {
        name: "HakanaiError",
        code: HakanaiErrorCodes.AUTHENTICATION_REQUIRED,
        message: "Auth required",
      };

      // Should not throw when onAuthenticationError is not defined
      expect(() => {
        handleAPIError(authError, "Fallback", basicHandler);
      }).not.toThrow();

      expect(basicHandler.displayError).toHaveBeenCalled();
    });

    test("should use original error message when translation not found", () => {
      // Mock i18n to return the key (no translation found)
      (window as any).i18n.t.mockImplementation((key: string) => key); // Return key as-is

      const hakanaiError = {
        name: "HakanaiError",
        code: "UNKNOWN_CODE",
        message: "Original error message",
      };

      handleAPIError(hakanaiError, "Fallback", mockHandler);

      expect(mockHandler.displayError).toHaveBeenCalledWith(
        "Original error message",
      );
    });
  });
});
