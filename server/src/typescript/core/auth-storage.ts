// SPDX-License-Identifier: MIT

// SessionStorage management for auth tokens
const AUTH_TOKEN_KEY = "hakanai-auth-token";

/**
 * Save authentication token to session storage
 * @param token - Auth token to store
 * @returns True if successful, false otherwise
 */
export function saveAuthTokenToStorage(token: string): boolean {
  if (!token.trim()) return false;

  try {
    sessionStorage.setItem(AUTH_TOKEN_KEY, token);
    return true;
  } catch (error) {
    console.warn("Failed to save auth token to sessionStorage:", error);
    return false;
  }
}

/**
 * Retrieve authentication token from session storage
 * @returns Stored token or null if not found/error
 */
export function getAuthTokenFromStorage(): string | null {
  try {
    return sessionStorage.getItem(AUTH_TOKEN_KEY);
  } catch (error) {
    console.warn("Failed to read auth token from sessionStorage:", error);
    return null;
  }
}

/**
 * Clear authentication token from session storage
 */
export function clearAuthTokenStorage(): void {
  try {
    sessionStorage.removeItem(AUTH_TOKEN_KEY);
  } catch (error) {
    console.warn("Failed to clear auth token from sessionStorage:", error);
  }
}
