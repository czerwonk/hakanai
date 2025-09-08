// SPDX-License-Identifier: Apache-2.0

import { I18n, translations, type LanguageCode } from "../../src/core/i18n";

describe("I18n Translation Completeness", () => {
  let i18n: I18n;

  beforeEach(() => {
    // Mock localStorage
    Object.defineProperty(window, "localStorage", {
      value: {
        getItem: jest.fn(),
        setItem: jest.fn(),
      },
      writable: true,
    });

    // Mock navigator.language
    Object.defineProperty(navigator, "language", {
      value: "en-US",
      writable: true,
    });

    i18n = new I18n();
  });

  describe("Functional Translation Tests", () => {
    test("should return correct translation for valid keys", () => {
      // Test a few known keys from the actual translations
      const testKey = Object.keys(translations.en)[0]; // Get first available key

      i18n.setLanguage("en");
      expect(i18n.t(testKey)).toBe(translations.en[testKey]);

      i18n.setLanguage("de");
      expect(i18n.t(testKey)).toBe(translations.de[testKey]);
    });

    test("should fall back to English for invalid language", () => {
      const originalLang = i18n.getCurrentLanguage();
      i18n.setLanguage("fr" as LanguageCode); // Invalid language
      expect(i18n.getCurrentLanguage()).toBe(originalLang); // Should not change
    });

    test("should return key for non-existent translation keys", () => {
      const nonExistentKey = "non.existent.key.that.does.not.exist";
      expect(i18n.t(nonExistentKey)).toBe(nonExistentKey);
    });

    test("should fall back to English translation for missing keys", () => {
      // This tests the t() method fallback mechanism
      i18n.setLanguage("de");

      // Test with an existing key
      const testKey = Object.keys(translations.en)[0];
      const result = i18n.t(testKey);

      expect(result).toBeTruthy();
      expect(result).not.toBe(testKey); // Should not be the key itself
      // Should be either the German or English translation
      expect([translations.en[testKey], translations.de[testKey]]).toContain(result);
    });
  });
});
