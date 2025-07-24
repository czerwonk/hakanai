import typescript from "@rollup/plugin-typescript";

export default [
  // Bundled files - include all dependencies and translations
  {
    input: "server/src/typescript/create-secret.ts",
    output: {
      file: "server/src/includes/create-secret.js",
      format: "iife",
      name: "CreateSecret",
    },
    plugins: [
      typescript({
        tsconfig: "./tsconfig.json",
        sourceMap: false,
        declaration: false,
      }),
    ],
    onwarn: (warning, warn) => {
      if (
        warning.plugin === "typescript" &&
        warning.message.includes("hakanai_wasm.js")
      ) {
        return;
      }
      warn(warning);
    },
  },
  {
    input: "server/src/typescript/get-secret.ts",
    output: {
      file: "server/src/includes/get-secret.js",
      format: "iife",
      name: "GetSecret",
    },
    plugins: [
      typescript({
        tsconfig: "./tsconfig.json",
        sourceMap: false,
        declaration: false,
      }),
    ],
    onwarn: (warning, warn) => {
      if (
        warning.plugin === "typescript" &&
        warning.message.includes("hakanai_wasm.js")
      ) {
        return;
      }
      warn(warning);
    },
  },
  {
    input: "server/src/typescript/share.ts",
    output: {
      file: "server/src/includes/share.js",
      format: "iife",
      name: "Share",
    },
    plugins: [
      typescript({
        tsconfig: "./tsconfig.json",
        sourceMap: false,
        declaration: false,
      }),
    ],
    onwarn: (warning, warn) => {
      if (
        warning.plugin === "typescript" &&
        warning.message.includes("hakanai_wasm.js")
      ) {
        return;
      }
      warn(warning);
    },
  },

  // Common bundle for static pages (homepage, impressum, privacy, docs)
  {
    input: "server/src/typescript/common.ts",
    output: {
      file: "server/src/includes/common.js",
      format: "iife",
      name: "Common",
    },
    plugins: [
      typescript({
        tsconfig: "./tsconfig.json",
        sourceMap: false,
        declaration: false,
      }),
    ],
    onwarn: (warning, warn) => {
      if (
        warning.plugin === "typescript" &&
        warning.message.includes("hakanai_wasm.js")
      ) {
        return;
      }
      warn(warning);
    },
  },

  // Standalone client library - no bundling, readable output
  {
    input: "server/src/typescript/hakanai-client.ts",
    output: {
      file: "server/src/includes/hakanai-client.js",
      format: "iife",
      name: "HakanaiClient",
    },
    plugins: [
      typescript({
        tsconfig: "./tsconfig.json",
        sourceMap: false,
        declaration: false,
      }),
    ],
    onwarn: (warning, warn) => {
      if (
        warning.plugin === "typescript" &&
        warning.message.includes("hakanai_wasm.js")
      ) {
        return;
      }
      warn(warning);
    },
  },
];
