// ESLint for the mockup catalogue. The sources live under docs/mockups, so the
// lint script runs from the repository root with this file as --config, and
// every pattern below is relative to the repository root.
import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores(["docs/mockups/mokly-generated/**"], "mockups/generated"),
  {
    name: "mockups/sources",
    files: ["docs/mockups/**/*.{ts,tsx}"],
    extends: [js.configs.recommended, tseslint.configs.recommended],
  },
  {
    name: "mockups/package",
    files: ["ts/mockups/eslint.config.js"],
    extends: [js.configs.recommended],
  },
]);
