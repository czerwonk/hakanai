// SPDX-License-Identifier: Apache-2.0

import { encode, decode } from "@msgpack/msgpack";
import { HakanaiError, HakanaiErrorCodes } from "./errors";

interface PayloadData {
  readonly data: Uint8Array;
  readonly filename?: string;

  /**
   * Set data from raw bytes (for binary files or text converted to bytes)
   */
  setFromBytes(bytes: ArrayBuffer): void;

  /**
   * Set data directly from base64-encoded string (optimization for pre-encoded data)
   */
  setFromBase64(base64Data: string): void;

  /**
   * Return the payload data as a UTF-8 string
   */
  text(): string;

  /*
   * Return the raw bytes of the payload data
   */
  bytes(): Uint8Array;

  /*
   * Set the filename associated with the payload (optional)
   */
  setFilename(filename: string): void;

  /**
   * Serialize the payload to MessagePack format matching Rust's rmp_serde
   */
  serialize(): Uint8Array;
}

/**
 * PayloadData implementation class
 */
class PayloadDataImpl implements PayloadData {
  private _data: Uint8Array = new Uint8Array();
  private _filename?: string;

  constructor(data: Uint8Array = new Uint8Array(), filename?: string) {
    this._data = data;
    this._filename = filename;
  }

  get data(): Uint8Array {
    return this._data;
  }

  get filename(): string | undefined {
    return this._filename;
  }

  setFromBytes(bytes: ArrayBuffer): void {
    this._data = new Uint8Array(bytes);
  }

  setFromBase64(base64Data: string): void {
    if (typeof base64Data !== "string") {
      throw new HakanaiError(HakanaiErrorCodes.EXPECTED_STRING, "Base64 data must be a string");
    }

    // Validate base64 format (basic check)
    if (!/^[A-Za-z0-9+/]*={0,2}$/.test(base64Data)) {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_INPUT_FORMAT, "Invalid base64 format");
    }

    const binaryString = atob(base64Data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    this._data = bytes;
  }

  text(): string {
    const decoder = new TextDecoder();
    return decoder.decode(this._data.buffer);
  }

  bytes(): Uint8Array {
    return this._data;
  }

  setFilename(filename: string): void {
    this._filename = filename;
  }

  /**
   * Serialize the payload to MessagePack format.
   * The payload is serialized as a 2-element array: [data, filename]
   */
  serialize(): Uint8Array {
    const payload: [Uint8Array, string | null] = [this._data, this._filename ?? null];
    return new Uint8Array(encode(payload));
  }

  /**
   * Deserialize a MessagePack payload.
   *
   * @param bytes - MessagePack-encoded bytes
   * @returns PayloadDataImpl instance
   * @throws {HakanaiError} If deserialization fails or format is invalid
   */
  static deserialize(bytes: ArrayBuffer): PayloadDataImpl {
    let decoded: unknown;
    try {
      decoded = decode(bytes);
    } catch {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_PAYLOAD, "Failed to decode MessagePack payload");
    }

    // Validate the decoded structure is a 2-element array
    if (!Array.isArray(decoded) || decoded.length !== 2) {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_PAYLOAD, "Invalid payload structure: expected 2-element array");
    }

    const [data, filename] = decoded;

    // Validate data is Uint8Array or can be converted
    if (!Array.isArray(data)) {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_PAYLOAD, "Invalid payload: data must be binary");
    }

    // Validate filename is string or null
    if (filename !== null && typeof filename !== "string") {
      throw new HakanaiError(HakanaiErrorCodes.INVALID_PAYLOAD, "Invalid payload: filename must be string or null");
    }

    return new PayloadDataImpl(new Uint8Array(data), filename ?? undefined);
  }
}

export { type PayloadData, PayloadDataImpl };
