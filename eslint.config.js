import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";
import pluginReactConfig from "eslint-plugin-react/configs/recommended.js";

export default [
  {
    settings: {
      react: {
        version: "detect",
      },
    },
  },
  {
    ignores: ["pkg/*"],
  },
  { languageOptions: { globals: globals.browser } },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
  {
    ...pluginReactConfig,
    rules: { ...pluginReactConfig.rules, "react/react-in-jsx-scope": 0 },
  },
].map((x) => {
  if (!!x.rules && !!x.rules["@typescript-eslint/no-explicit-any"]) {
    x.rules["@typescript-eslint/no-explicit-any"] = 0;
  }
  return x;
});
