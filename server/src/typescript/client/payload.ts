// SPDX-License-Identifier: Apache-2.0

import { HakanaiError, HakanaiErrorCodes } from "./errors";

interface PayloadData {
  readonly data: string;
  readonly filename?: string;

  /**
   * Set data from raw bytes (for binary files or text converted to bytes)
   */
  setFromBytes(bytes: Uint8Array): void;

  /**
   * Set data directly from base64-encoded string (optimization for pre-encoded data)
   */
  setFromBase64(base64Data: string): void;

  /**
   * Decode the base64-encoded data field to a readable string
   */
  decode(): string;

  /**
   * Decode the base64-encoded data field to bytes for binary data
   */
  decodeBytes(): Uint8Array;
}

/**
 * PayloadData implementation class
 */
class PayloadDataImpl implements PayloadData {
  private _data: string = "";
  private _filename?: string;

  constructor(data: string = "", filename?: string) {
    this._data = data;
    this._filename = filename;
  }

  get data(): string {
    return this._data;
  }

  get filename(): string | undefined {
    return this._filename;
  }

  setFromBytes(bytes: Uint8Array): void {
    if (!(bytes instanceof Uint8Array)) {
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_UINT8_ARRAY,
        "Data must be a Uint8Array",
      );
    }

    // Convert bytes to base64 for storage
    let binaryString = "";
    const chunkSize = 8192;

    for (let i = 0; i < bytes.length; i += chunkSize) {
      const chunk = bytes.subarray(i, i + chunkSize);
      binaryString += String.fromCharCode(...chunk);
    }

    this._data = btoa(binaryString);
  }

  setFromBase64(base64Data: string): void {
    if (typeof base64Data !== "string") {
      throw new HakanaiError(
        HakanaiErrorCodes.EXPECTED_STRING,
        "Base64 data must be a string",
      );
    }

    // Validate base64 format (basic check)
    if (!/^[A-Za-z0-9+/]*={0,2}$/.test(base64Data)) {
      throw new HakanaiError(
        HakanaiErrorCodes.INVALID_INPUT_FORMAT,
        "Invalid base64 format",
      );
    }

    this._data = base64Data;
  }

  decode(): string {
    const decoder = new TextDecoder();
    return decoder.decode(this.decodeBytes());
  }

  decodeBytes(): Uint8Array {
    const binaryString = atob(this._data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
  }
}

export { type PayloadData, PayloadDataImpl };

