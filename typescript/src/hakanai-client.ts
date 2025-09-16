// SPDX-License-Identifier: Apache-2.0

/*
 * Hakanai JavaScript Client (TypeScript Implementation)
 *
 * This client implements the same cryptographic protocol as the Rust CLI client,
 * allowing you to send and receive encrypted secrets via the Hakanai API.
 *
 * This is the main entry point that re-exports all client functionality.
 */

// Re-export everything from the modular client components
export { HakanaiError, HakanaiErrorCodes } from "./client/errors";
export { InputValidation } from "./client/validation";
export { UrlParser } from "./client/url-parser";
export { type CompatibilityCheck, BrowserCompatibility } from "./client/browser-compat";
export { Base64UrlSafe } from "./client/base64-utils";
export { ContentAnalysis } from "./client/content-analysis";
export { HashUtils } from "./client/hash-utils";
export { CryptoContext } from "./client/crypto-operations";
export { SecureMemory } from "./client/secure-memory";
export { type PayloadData, PayloadDataType, PayloadDataImpl } from "./client/payload";
export { type DataTransferObserver } from "./client/progress-observer";
export { HakanaiClient, SecretRequest, SecretResponse, SecretRestrictions } from "./client/client";
