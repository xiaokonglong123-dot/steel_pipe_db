import js from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';

// eslint-plugin-react-hooks must be installed for these rules to activate.
// Run: npm install -D eslint-plugin-react-hooks
let reactHooksPlugin;
try {
  reactHooksPlugin = (await import('eslint-plugin-react-hooks')).default;
} catch {
  // Plugin not installed yet — rules are skipped.
}

const configs = [
  {
    ignores: ['dist/**', 'node_modules/**'],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      globals: {
        ...globals.browser,
      },
    },
  },
];

if (reactHooksPlugin) {
  configs.push({
    plugins: {
      'react-hooks': reactHooksPlugin,
    },
    rules: {
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn',
    },
  });
}

export default tseslint.config(...configs);
