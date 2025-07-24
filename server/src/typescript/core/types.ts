// Error Types
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

// ShareData Types

/**
 * Validation error codes for ShareData
 */
export enum ShareDataValidationError {
  MISSING_DATA = "MISSING_DATA",
  INVALID_FILENAME = "INVALID_FILENAME",
  INVALID_TOKEN = "INVALID_TOKEN",
  INVALID_TTL = "INVALID_TTL",
  EMPTY_JSON = "EMPTY_JSON",
  INVALID_JSON_FORMAT = "INVALID_JSON_FORMAT",
}

/**
 * Custom error class for ShareData validation
 */
export class ShareDataError extends Error {
  constructor(
    public readonly code: ShareDataValidationError,
    message: string,
  ) {
    super(message);
    this.name = "ShareDataError";
  }
}

/**
 * Share data structure for clipboard and fragment-based sharing
 */
export class ShareData {
  constructor(
    public readonly data: string, // base64-encoded content
    public readonly filename?: string,
    public readonly token?: string,
    public readonly ttl?: number,
  ) {
    this.validate();
  }

  /**
   * Validate the share data
   * @throws Error if validation fails
   */
  private validate(): void {
    // Validate required fields
    if (!this.data || typeof this.data !== "string") {
      throw new ShareDataError(
        ShareDataValidationError.MISSING_DATA,
        'Missing or invalid "data" field',
      );
    }

    // Validate optional fields
    if (this.filename != null && typeof this.filename !== "string") {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_FILENAME,
        'Invalid "filename" field - must be string',
      );
    }

    if (this.token != null && typeof this.token !== "string") {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_TOKEN,
        'Invalid "token" field - must be string',
      );
    }

    if (
      this.ttl !== undefined &&
      (typeof this.ttl !== "number" || this.ttl <= 0 || isNaN(this.ttl))
    ) {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_TTL,
        'Invalid "ttl" field - must be positive number',
      );
    }
  }

  /**
   * Create ShareData from JSON string (clipboard content)
   * @param jsonString JSON string containing share data
   * @returns ShareData instance
   * @throws Error if JSON is invalid or validation fails
   */
  static fromJSON(jsonString: string): ShareData {
    if (!jsonString.trim()) {
      throw new ShareDataError(
        ShareDataValidationError.EMPTY_JSON,
        "JSON string is empty",
      );
    }

    let payload;
    try {
      payload = JSON.parse(jsonString);
    } catch (error) {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_JSON_FORMAT,
        "Invalid JSON format",
      );
    }

    return new ShareData(
      payload.data,
      payload.filename,
      payload.token,
      payload.ttl,
    );
  }

  /**
   * Create ShareData from URL fragment parameters
   * @param fragment URL fragment (without #)
   * @returns ShareData instance or null if no data found
   * @throws Error if validation fails
   */
  static fromFragment(fragment: string): ShareData | null {
    if (!fragment) return null;

    const params = new URLSearchParams(fragment);
    const data = params.get("data");

    if (!data) return null;

    return new ShareData(
      data,
      params.get("filename") ?? undefined,
      params.get("token") ?? undefined,
      params.get("ttl") ? parseInt(params.get("ttl")!) : undefined,
    );
  }

  /**
   * Calculate content size in bytes from base64 data
   */
  getContentSize(): number {
    return Math.ceil((this.data.length * 3) / 4);
  }
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
