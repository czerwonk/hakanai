import { HakanaiError, HakanaiErrorCodes } from "../hakanai-client.js";

// Error handler interface - UI classes implement this
export interface ErrorHandler {
  /**
   * Display an error message to the user
   * @param message - The error message to display
   */
  displayError(message: string): void;

  /**
   * Called when an authentication error occurs
   * Optional - only implement if special handling is needed
   */
  onAuthenticationError?(): void;
}

interface StandardError extends Error {
  readonly name: string;
  readonly message: string;
}

interface UnknownError {
  readonly message?: string;
  readonly name?: string;
  readonly code?: string;
}

// Type guards for error handling

/**
 * Type guard to check if an error is a HakanaiError with error code
 * @param error - Unknown error to check
 * @returns True if error is HakanaiError with valid structure
 */
function isHakanaiError(error: unknown): error is HakanaiError {
  return (
    typeof error === "object" &&
    error !== null &&
    "name" in error &&
    (error as { name: unknown }).name === "HakanaiError" &&
    "code" in error &&
    typeof (error as { code: unknown }).code === "string"
  );
}

/**
 * Type guard to check if an error is a standard Error instance
 * @param error - Unknown error to check
 * @returns True if error is an Error instance
 */
function isStandardError(error: unknown): error is StandardError {
  return error instanceof Error;
}

/**
 * Type guard to check if an object has error-like properties
 * @param error - Unknown value to check
 * @returns True if object has message or name properties
 */
function isErrorLike(error: unknown): error is UnknownError {
  return (
    typeof error === "object" &&
    error !== null &&
    ("message" in error || "name" in error)
  );
}

/**
 * Get localized message for HakanaiError
 * @param error - The HakanaiError to get message for
 * @returns Localized message or fallback to error's message
 */
function getHakanaiErrorMessage(error: HakanaiError): string {
  const errorKey = `error.${error.code}`;
  const localizedMessage = window.i18n.t(errorKey);
  return localizedMessage !== errorKey ? localizedMessage : error.message;
}

/**
 * Process an API error and delegate to the appropriate handler
 * @param error - The error from API call
 * @param fallbackMessage - Message to show for unknown errors
 * @param handler - The UI handler that will display the error
 */
export function handleAPIError(
  error: unknown,
  fallbackMessage: string,
  handler: ErrorHandler,
): void {
  let message: string;

  if (isHakanaiError(error)) {
    message = getHakanaiErrorMessage(error);

    if (
      error.code === HakanaiErrorCodes.AUTHENTICATION_REQUIRED ||
      error.code === HakanaiErrorCodes.INVALID_TOKEN
    ) {
      handler.onAuthenticationError?.();
    }
  } else if (isStandardError(error)) {
    message = error.message;
  } else if (isErrorLike(error)) {
    message = error.message ?? fallbackMessage;
  } else {
    message = fallbackMessage;
  }

  handler.displayError(message);
}

