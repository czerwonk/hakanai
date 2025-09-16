// SPDX-License-Identifier: Apache-2.0

import { RestrictionData } from "./restriction-data.js";
import { PayloadDataType } from "../hakanai-client.js";

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
  INVALID_DATA_TYPE = "INVALID_DATA_TYPE",
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
  public readonly data_type?: PayloadDataType;

  constructor(
    public readonly data: string, // base64-encoded content
    public readonly filename?: string,
    public readonly token?: string,
    public readonly ttl?: number,
    public readonly restrictions?: RestrictionData,
    data_type?: PayloadDataType | string, // Accept string in constructor for parsing
  ) {
    if (data_type !== undefined) {
      this.data_type = this.parseDataType(data_type);
    }

    this.validate();
  }

  /**
   * Parse and validate data_type string to enum
   */
  private parseDataType(value: PayloadDataType | string): PayloadDataType {
    if (typeof value !== "string") {
      return value;
    }

    const validTypes = Object.values(PayloadDataType);
    if (!validTypes.includes(value as PayloadDataType)) {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_DATA_TYPE,
        `Invalid "data_type" value "${value}" - must be one of: ${validTypes.join(", ")}`,
      );
    }

    return value as PayloadDataType;
  }

  /**
   * Validate the share data
   * @throws Error if validation fails
   */
  private validate(): void {
    // Validate required fields
    if (!this.data || typeof this.data !== "string") {
      throw new ShareDataError(ShareDataValidationError.MISSING_DATA, 'Missing or invalid "data" field');
    }

    // Validate optional fields
    if (this.filename != null && typeof this.filename !== "string") {
      throw new ShareDataError(ShareDataValidationError.INVALID_FILENAME, 'Invalid "filename" field - must be string');
    }

    if (this.token != null && typeof this.token !== "string") {
      throw new ShareDataError(ShareDataValidationError.INVALID_TOKEN, 'Invalid "token" field - must be string');
    }

    if (this.ttl !== undefined && (typeof this.ttl !== "number" || this.ttl <= 0 || isNaN(this.ttl))) {
      throw new ShareDataError(ShareDataValidationError.INVALID_TTL, 'Invalid "ttl" field - must be positive number');
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
      throw new ShareDataError(ShareDataValidationError.EMPTY_JSON, "JSON string is empty");
    }

    let payload;
    try {
      payload = JSON.parse(jsonString);
    } catch (error) {
      throw new ShareDataError(ShareDataValidationError.INVALID_JSON_FORMAT, "Invalid JSON format");
    }

    return new ShareData(
      payload.data,
      payload.filename,
      payload.token,
      payload.ttl,
      payload.restrictions,
      payload.data_type,
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
      undefined, // restrictions are not passed via fragment
      params.get("data_type") ?? undefined,
    );
  }

  /**
   * Calculate content size in bytes from base64 data
   */
  getContentSize(): number {
    return Math.ceil((this.data.length * 3) / 4);
  }
}
