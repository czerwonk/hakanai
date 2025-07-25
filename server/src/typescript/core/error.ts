import { HakanaiError } from "../hakanai-client.js";

export interface StandardError extends Error {
  readonly name: string;
  readonly message: string;
}

export interface UnknownError {
  readonly message?: string;
  readonly name?: string;
  readonly code?: string;
}

export type AppError = HakanaiError | StandardError | UnknownError;

// Type guards for error handling

/**
 * Type guard to check if an error is a HakanaiError with error code
 * @param error - Unknown error to check
 * @returns True if error is HakanaiError with valid structure
 */
export function isHakanaiError(error: unknown): error is HakanaiError {
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
export function isStandardError(error: unknown): error is StandardError {
  return error instanceof Error;
}

/**
 * Type guard to check if an object has error-like properties
 * @param error - Unknown value to check
 * @returns True if object has message or name properties
 */
export function isErrorLike(error: unknown): error is UnknownError {
  return (
    typeof error === "object" &&
    error !== null &&
    ("message" in error || "name" in error)
  );
}
