// Type guards for error handling
/**
 * Type guard to check if an error is a HakanaiError with error code
 * @param error - Unknown error to check
 * @returns True if error is HakanaiError with valid structure
 */
export function isHakanaiError(error) {
    return (typeof error === "object" &&
        error !== null &&
        "name" in error &&
        error.name === "HakanaiError" &&
        "code" in error &&
        typeof error.code === "string");
}
/**
 * Type guard to check if an error is a standard Error instance
 * @param error - Unknown error to check
 * @returns True if error is an Error instance
 */
export function isStandardError(error) {
    return error instanceof Error;
}
/**
 * Type guard to check if an object has error-like properties
 * @param error - Unknown value to check
 * @returns True if object has message or name properties
 */
export function isErrorLike(error) {
    return (typeof error === "object" &&
        error !== null &&
        ("message" in error || "name" in error));
}
// ShareData Types
/**
 * Validation error codes for ShareData
 */
export var ShareDataValidationError;
(function (ShareDataValidationError) {
    ShareDataValidationError["MISSING_DATA"] = "MISSING_DATA";
    ShareDataValidationError["INVALID_FILENAME"] = "INVALID_FILENAME";
    ShareDataValidationError["INVALID_TOKEN"] = "INVALID_TOKEN";
    ShareDataValidationError["INVALID_TTL"] = "INVALID_TTL";
    ShareDataValidationError["EMPTY_JSON"] = "EMPTY_JSON";
    ShareDataValidationError["INVALID_JSON_FORMAT"] = "INVALID_JSON_FORMAT";
})(ShareDataValidationError || (ShareDataValidationError = {}));
/**
 * Custom error class for ShareData validation
 */
export class ShareDataError extends Error {
    constructor(code, message) {
        super(message);
        this.code = code;
        this.name = "ShareDataError";
    }
}
/**
 * Share data structure for clipboard and fragment-based sharing
 */
export class ShareData {
    constructor(data, // base64-encoded content
    filename, token, ttl) {
        this.data = data;
        this.filename = filename;
        this.token = token;
        this.ttl = ttl;
        this.validate();
    }
    /**
     * Validate the share data
     * @throws Error if validation fails
     */
    validate() {
        // Validate required fields
        if (!this.data || typeof this.data !== "string") {
            throw new ShareDataError(ShareDataValidationError.MISSING_DATA, 'Missing or invalid "data" field');
        }
        // Validate optional fields
        if (this.filename !== undefined && typeof this.filename !== "string") {
            throw new ShareDataError(ShareDataValidationError.INVALID_FILENAME, 'Invalid "filename" field - must be string');
        }
        if (this.token !== undefined && typeof this.token !== "string") {
            throw new ShareDataError(ShareDataValidationError.INVALID_TOKEN, 'Invalid "token" field - must be string');
        }
        if (this.ttl !== undefined &&
            (typeof this.ttl !== "number" || this.ttl <= 0 || isNaN(this.ttl))) {
            throw new ShareDataError(ShareDataValidationError.INVALID_TTL, 'Invalid "ttl" field - must be positive number');
        }
    }
    /**
     * Create ShareData from JSON string (clipboard content)
     * @param jsonString JSON string containing share data
     * @returns ShareData instance
     * @throws Error if JSON is invalid or validation fails
     */
    static fromJSON(jsonString) {
        if (!jsonString.trim()) {
            throw new ShareDataError(ShareDataValidationError.EMPTY_JSON, "JSON string is empty");
        }
        let payload;
        try {
            payload = JSON.parse(jsonString);
        }
        catch (error) {
            throw new ShareDataError(ShareDataValidationError.INVALID_JSON_FORMAT, "Invalid JSON format");
        }
        return new ShareData(payload.data, payload.filename, payload.token, payload.ttl);
    }
    /**
     * Create ShareData from URL fragment parameters
     * @param fragment URL fragment (without #)
     * @returns ShareData instance or null if no data found
     * @throws Error if validation fails
     */
    static fromFragment(fragment) {
        if (!fragment)
            return null;
        const params = new URLSearchParams(fragment);
        const data = params.get("data");
        if (!data)
            return null;
        return new ShareData(data, params.get("filename") || undefined, params.get("token") || undefined, params.get("ttl") ? parseInt(params.get("ttl")) : undefined);
    }
    /**
     * Calculate content size in bytes from base64 data
     */
    getContentSize() {
        return Math.ceil((this.data.length * 3) / 4);
    }
}
