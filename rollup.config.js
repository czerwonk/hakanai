import typescript from '@rollup/plugin-typescript';
import { nodeResolve } from '@rollup/plugin-node-resolve';

export default [
  // Bundled files - include all dependencies and translations
  {
    input: 'server/src/typescript/create-secret.ts',
    output: {
      file: 'server/src/includes/create-secret.js',
      format: 'iife',
      name: 'CreateSecret'
    },
    plugins: [
      nodeResolve(),
      typescript({
        tsconfig: './tsconfig.json',
        sourceMap: false,
        declaration: false
      })
    ]
  },
  {
    input: 'server/src/typescript/get-secret.ts',
    output: {
      file: 'server/src/includes/get-secret.js',
      format: 'iife',
      name: 'GetSecret'
    },
    plugins: [
      nodeResolve(),
      typescript({
        tsconfig: './tsconfig.json',
        sourceMap: false,
        declaration: false
      })
    ]
  },
  {
    input: 'server/src/typescript/share.ts',
    output: {
      file: 'server/src/includes/share.js',
      format: 'iife',
      name: 'Share'
    },
    plugins: [
      nodeResolve(),
      typescript({
        tsconfig: './tsconfig.json',
        sourceMap: false,
        declaration: false
      })
    ]
  },
  {
    input: 'server/src/typescript/docs.ts',
    output: {
      file: 'server/src/includes/docs.js',
      format: 'iife',
      name: 'Docs'
    },
    plugins: [
      nodeResolve(),
      typescript({
        tsconfig: './tsconfig.json',
        sourceMap: false,
        declaration: false
      })
    ]
  },

  // Common bundle for static pages (homepage, impressum, privacy)
  {
    input: 'server/src/typescript/common.ts',
    output: {
      file: 'server/src/includes/common.js',
      format: 'iife',
      name: 'Common'
    },
    plugins: [
      nodeResolve(),
      typescript({
        tsconfig: './tsconfig.json',
        sourceMap: false,
        declaration: false
      })
    ]
  },

  // Standalone client library - no bundling, readable output
  {
    input: 'server/src/typescript/hakanai-client.ts',
    output: {
      file: 'server/src/includes/hakanai-client.js',
      format: 'iife',
      name: 'HakanaiClient'
    },
    external: [], // No external dependencies expected
    plugins: [
      typescript({
        tsconfig: './tsconfig.json',
        sourceMap: false,
        declaration: false
      })
    ]
  }
];