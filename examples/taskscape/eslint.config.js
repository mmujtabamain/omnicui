import tseslint from "typescript-eslint";
import taskscape from "@taskscape/eslint-plugin";

export default [
  { ignores: ["**/node_modules/**", "**/dist/**"] },
  ...tseslint.configs.recommended.map((c) => ({ ...c, files: ["**/*.{ts,tsx}"] })),
  {
    // A leading underscore marks a deliberately unused name (a prop taken out of the spread).
    files: ["**/*.{ts,tsx}"],
    rules: { "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_", ignoreRestSiblings: true }] },
  },
  {
    // Harness entries take whatever props a contract test sets.
    files: ["**/src/harness/**/*.tsx"],
    rules: { "@typescript-eslint/no-explicit-any": "off" },
  },
  {
    // The apps: only widgets from @taskscape/ui, layout tags, and token utilities.
    files: ["apps/web/frontend/src/**/*.{ts,tsx}"],
    plugins: { taskscape },
    rules: { "taskscape/no-raw-elements": "error", "taskscape/no-ui-primitives": "error", "taskscape/no-raw-style": "error" },
  },
  {
    // The library may use raw elements and primitives (that is its job) but still no raw colours.
    files: ["packages/ui/src/components/**/*.tsx"],
    plugins: { taskscape },
    rules: { "taskscape/no-raw-style": ["error", { allowStyle: true }] },
  },
];
