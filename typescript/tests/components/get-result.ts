// SPDX-License-Identifier: Apache-2.0

import { generateFilename } from "../../src/components/get-result";

describe("get-result.ts", () => {
  describe("Filename generation", () => {
    test("generateFilename uses payload filename when available", () => {
      const payloadWithFilename = { filename: "document.pdf" };
      expect(generateFilename(payloadWithFilename, false)).toBe("document.pdf");
    });

    test("generateFilename creates timestamp filename when no filename", () => {
      const payloadWithoutFilename = { filename: undefined };
      const result = generateFilename(payloadWithoutFilename, false);

      expect(result).toMatch(/^hakanai-secret-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}.*\.txt$/);
    });

    test("generateFilename handles null filename", () => {
      const payloadWithNullFilename = { filename: null };
      const result = generateFilename(payloadWithNullFilename, false);

      expect(result).toMatch(/^hakanai-secret-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}.*\.txt$/);
    });

    test("generateFilename uses .bin extension for binary content", () => {
      const payloadWithoutFilename = { filename: undefined };
      const result = generateFilename(payloadWithoutFilename, true);

      expect(result).toMatch(/^hakanai-secret-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}.*\.bin$/);
    });

    test("generateFilename uses .txt extension for text content", () => {
      const payloadWithoutFilename = { filename: undefined };
      const result = generateFilename(payloadWithoutFilename, false);

      expect(result).toMatch(/^hakanai-secret-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}.*\.txt$/);
    });

    test("generateFilename prefers payload filename over binary detection", () => {
      const payloadWithFilename = { filename: "important.pdf" };
      const result = generateFilename(payloadWithFilename, true);

      expect(result).toBe("important.pdf");
    });
  });
});
