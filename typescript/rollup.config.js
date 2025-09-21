import typescript from "@rollup/plugin-typescript";

const commonConfig = {
  plugins: [
    typescript({
      tsconfig: "./tsconfig.json",
      sourceMap: false,
      declaration: false,
    }),
  ],
  onwarn: (warning, warn) => {
    if (warning.plugin === "typescript" && warning.message.includes("hakanai_wasm.js")) {
      return;
    }
    warn(warning);
  },
};

const createBundle = (fileName) => ({
  input: `src/${fileName}.ts`,
  output: {
    file: `../server/includes/${fileName}.js`,
    format: "es",
  },
  ...commonConfig,
});

const bundles = ["create-secret", "get-secret", "share", "common", "hakanai-client", "sw"];

export default bundles.map((fileName) => createBundle(fileName));
