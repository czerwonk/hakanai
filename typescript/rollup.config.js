import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";

const commonConfig = {
  plugins: [
    resolve(),
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

const bundles = ["create-secret", "get-secret", "share", "common", "sw", "one-time-token"];

export default bundles.map((fileName) => createBundle(fileName));
