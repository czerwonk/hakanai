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
    if (
      warning.plugin === "typescript" &&
      warning.message.includes("hakanai_wasm.js")
    ) {
      return;
    }
    warn(warning);
  },
};

const createBundle = (inputFile, outputFile) => ({
  input: `server/src/typescript/${inputFile}.ts`,
  output: {
    file: `server/src/includes/${outputFile}.js`,
    format: "es",
  },
  ...commonConfig,
});

const bundles = [
  ["create-secret", "create-secret"],
  ["get-secret", "get-secret"],
  ["share", "share"],
  ["common", "common"],
  ["hakanai-client", "hakanai-client"],
];

export default bundles.map(([input, output]) => createBundle(input, output));
