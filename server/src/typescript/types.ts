// Error Types

/**
 * Custom error type for Hakanai-specific errors with error codes
 * @interface HakanaiError
 * @extends {Error}
 */
export interface HakanaiError extends Error {
  readonly name: "HakanaiError";
  readonly code: string;
  readonly statusCode?: number;
}

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

// DOM Element Types
export interface RequiredElements {
  loadingDiv: HTMLElement;
  button: HTMLButtonElement;
  secretInput: HTMLInputElement;
  fileInput: HTMLInputElement;
  authTokenInput: HTMLInputElement;
  ttlSelect: HTMLSelectElement;
  textRadio: HTMLInputElement;
  fileRadio: HTMLInputElement;
  resultDiv: HTMLElement;
}

export interface FileElements {
  fileInput: HTMLInputElement;
  fileInfoDiv: HTMLElement;
  fileNameSpan: HTMLElement;
  fileSizeSpan: HTMLElement;
  radioGroup: HTMLElement;
  textInputGroup: HTMLElement;
  fileInputGroup: HTMLElement;
  fileRadio: HTMLInputElement;
  textRadio: HTMLInputElement;
}

export interface GetSecretElements {
  urlInput: HTMLInputElement;
  keyInput: HTMLInputElement;
  keyInputGroup: HTMLElement;
  resultDiv: HTMLElement;
  loadingDiv: HTMLElement;
  button: HTMLButtonElement;
}

// Event Handler Types
export type ClickHandler = (event: MouseEvent) => void;
export type ChangeHandler = (event: Event) => void;
export type SubmitHandler = (event: SubmitEvent) => void;

// Crypto Types
export interface CryptoKey {
  readonly bytes: Uint8Array;
  readonly length: number;
}

/**
 * Represents secret payload data with optional file metadata
 * @interface PayloadData
 */
export interface PayloadData {
  /** Base64-encoded data */
  readonly data: string;
  /** Optional filename for file payloads */
  readonly filename?: string;
  /** Set data from raw bytes (automatically base64 encodes) */
  setFromBytes?(bytes: Uint8Array): void;
  /** Decode data as UTF-8 text string */
  decode?(): string;
  /** Decode data as raw bytes */
  decodeBytes?(): Uint8Array;
}

// API Types
export interface SecretRequest {
  data: string;
  expires_in: number;
}

export interface SecretResponse {
  id: string;
}

// Theme Types
export type ThemeMode = "light" | "dark";
export type LanguageCode = "en" | "de";

// UI State Types
export interface FormValues {
  authToken: string;
  ttl: number;
  isFileMode: boolean;
}

// File Handling Types
export interface FileValidationResult {
  isValid: boolean;
  error?: string;
}

// Storage Types
export interface StorageWrapper {
  getItem(key: string): string | null;
  setItem(key: string, value: string): boolean;
}

// Utility Types
export type NonNullable<T> = T extends null | undefined ? never : T;
export type ElementGetter<T extends HTMLElement> = () => T | null;

// Result Pattern for Error Handling

/**
 * Result pattern for type-safe error handling
 * @template T - Success data type
 * @template E - Error type (defaults to AppError)
 */
export type Result<T, E = AppError> =
  | { success: true; data: T }
  | { success: false; error: E };

// Async Operation Types
export type AsyncOperation<T> = () => Promise<T>;
export type ErrorHandler = (error: AppError) => void;

// Constants Types
export interface CryptoConstants {
  readonly KEY_LENGTH: 32;
  readonly NONCE_LENGTH: 12;
  readonly MAX_FILE_SIZE: number;
}

export interface UITimeouts {
  readonly DEBOUNCE: number;
  readonly CLEANUP_DELAY: number;
  readonly COPY_FEEDBACK: number;
  readonly SCREEN_READER_ANNOUNCEMENT: number;
}

export interface StorageKeys {
  readonly THEME: string;
  readonly LANGUAGE: string;
}
