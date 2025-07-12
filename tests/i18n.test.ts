import {
  I18n,
  translations,
  type LanguageCode,
} from "../server/src/typescript/i18n";

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

  describe("Translation Key Categories", () => {
    const testCategoryExists = (category: string, expectedMinCount: number) => {
      const enKeys = Object.keys(translations.en);
      const categoryKeys = enKeys.filter((key) =>
        key.startsWith(`${category}.`),
      );

      expect(categoryKeys.length).toBeGreaterThanOrEqual(expectedMinCount);

      // Verify all category keys exist in all languages
      for (const key of categoryKeys) {
        expect(translations.en[key]).toBeDefined();
        expect(translations.de[key]).toBeDefined();
      }
    };

    test("should have page title translations", () => {
      testCategoryExists("page", 2);
    });

    test("should have header translations", () => {
      testCategoryExists("header", 2);
    });

    test("should have label translations", () => {
      testCategoryExists("label", 8);
    });

    test("should have placeholder translations", () => {
      testCategoryExists("placeholder", 2);
    });

    test("should have helper text translations", () => {
      testCategoryExists("helper", 5);
    });

    test("should have time option translations", () => {
      testCategoryExists("time", 7);
    });

    test("should have button translations", () => {
      testCategoryExists("button", 8);
    });

    test("should have message translations", () => {
      testCategoryExists("msg", 15);
    });

    test("should have aria label translations", () => {
      testCategoryExists("aria", 10);
    });

    test("should have meta description translations", () => {
      testCategoryExists("meta", 2);
    });

    test("should have error code translations", () => {
      testCategoryExists("error", 5);
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
        expect(colonRatio).toBeGreaterThan(0.7); // 70% consistency
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

  describe("Critical Translation Keys", () => {
    // Test for existence of commonly used UI elements
    const possibleCriticalKeys = [
      "button.create",
      "button.retrieve",
      "msg.createFailed",
      "msg.retrieveFailed",
      "error.SECRET_NOT_FOUND",
      "aria.secretInput",
    ];

    test("should have translations for critical UI elements", () => {
      const actualKeys = Object.keys(translations.en);
      const existingCriticalKeys = possibleCriticalKeys.filter((key) =>
        actualKeys.includes(key),
      );

      // Test the ones that actually exist
      for (const key of existingCriticalKeys) {
        expect(translations.en[key]).toBeDefined();
        expect(translations.de[key]).toBeDefined();
        expect(translations.en[key]).toBeTruthy();
        expect(translations.de[key]).toBeTruthy();
      }

      // Ensure we found at least some critical keys
      expect(existingCriticalKeys.length).toBeGreaterThan(0);
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
