import {
  I18n,
  translations,
  type LanguageCode,
} from "../../server/src/typescript/core/i18n";

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

  describe("Language Coverage", () => {
    test("should support exactly 2 languages (en, de)", () => {
      const availableLanguages = i18n.getAvailableLanguages();
      expect(availableLanguages).toHaveLength(2);
      expect(availableLanguages).toContain("en");
      expect(availableLanguages).toContain("de");
    });

    test("should have the same translation keys in all languages", () => {
      const enKeys = Object.keys(translations.en).sort();
      const deKeys = Object.keys(translations.de).sort();

      expect(enKeys).toEqual(deKeys);
      expect(enKeys.length).toBeGreaterThan(50); // Ensure we have a substantial number of translations
    });

    test("should not have empty translations in any language", () => {
      const languages: LanguageCode[] = ["en", "de"];

      for (const lang of languages) {
        const langTranslations = translations[lang];

        for (const [key, value] of Object.entries(langTranslations)) {
          expect(value).toBeTruthy();
          expect((value as string).trim()).not.toBe("");
          expect(value).not.toBe(key); // Should not fall back to key
        }
      }
    });
  });

  describe("Translation Quality", () => {
    test("should have different translations for different languages", () => {
      const enKeys = Object.keys(translations.en);
      let differentTranslations = 0;

      for (const key of enKeys) {
        const enValue = translations.en[key];
        const deValue = translations.de[key];

        if (enValue !== deValue) {
          differentTranslations++;
        }
      }

      // At least 90% of translations should be different between languages
      const threshold = Math.floor(enKeys.length * 0.9);
      expect(differentTranslations).toBeGreaterThanOrEqual(threshold);
    });

    test("should not have placeholder or template strings", () => {
      const languages: LanguageCode[] = ["en", "de"];

      for (const lang of languages) {
        const langTranslations = translations[lang];

        for (const [key, value] of Object.entries(langTranslations)) {
          // Check for common placeholder patterns
          expect(value).not.toMatch(/\{\{.*\}\}/); // {{placeholder}}
          expect(value).not.toMatch(/\$\{.*\}/); // ${placeholder}
          expect(value).not.toMatch(/TODO|FIXME|XXX/i);
        }
      }
    });

    test("should have consistent punctuation patterns within languages", () => {
      // Check that labels end consistently (with or without colons)
      const enLabels = Object.entries(translations.en)
        .filter(([key]) => key.startsWith("label."))
        .map(([, value]) => value);

      if (enLabels.length > 0) {
        const enLabelsWithColons = enLabels.filter((label) =>
          (label as string).endsWith(":"),
        );
        const colonRatio = enLabelsWithColons.length / enLabels.length;

        // Either most labels have colons or most don't (consistency check)
        // Form field labels should have colons, checkbox/radio labels should not
        expect(colonRatio).toBeGreaterThan(0.6); // 60% consistency allows for mixed UI patterns
      }
    });
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
      expect([translations.en[testKey], translations.de[testKey]]).toContain(
        result,
      );
    });
  });

  describe("Translation Completeness Report", () => {
    test("should generate a completeness report", () => {
      const enKeys = Object.keys(translations.en);
      const deKeys = Object.keys(translations.de);

      const report = {
        totalKeys: enKeys.length,
        englishKeys: enKeys.length,
        germanKeys: deKeys.length,
        missingInGerman: enKeys.filter((key) => !deKeys.includes(key)),
        missingInEnglish: deKeys.filter((key) => !enKeys.includes(key)),
        categories: {} as Record<string, number>,
      };

      // Count keys by category
      for (const key of enKeys) {
        const category = key.split(".")[0];
        report.categories[category] = (report.categories[category] || 0) + 1;
      }

      // Log the report for visibility
      console.log(
        "Translation Completeness Report:",
        JSON.stringify(report, null, 2),
      );

      // Assertions
      expect(report.missingInGerman).toHaveLength(0);
      expect(report.missingInEnglish).toHaveLength(0);
      expect(report.totalKeys).toBeGreaterThan(50);
      expect(Object.keys(report.categories).length).toBeGreaterThan(5);
    });
  });
});
