import js from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';

const noColorLiteralsRule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow color literals outside tokens.css',
      category: 'Best Practices',
    },
    messages: {
      noColorLiteral:
        'Colour literal "{{value}}" forbidden. Use design tokens from tokens.css (AD-10, NFR-17).',
    },
    schema: [],
  },
  create(context) {
    const HEX_COLOR_REGEX = /#(?:[0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b/;
    const COLOR_FN_REGEX = /\b(?:rgba?|hsla?)\s*\(/;

    function checkString(node, str) {
      if (typeof str !== 'string') return;
      const hexMatch = str.match(HEX_COLOR_REGEX);
      if (hexMatch) {
        context.report({
          node,
          messageId: 'noColorLiteral',
          data: { value: hexMatch[0] },
        });
        return;
      }
      const fnMatch = str.match(COLOR_FN_REGEX);
      if (fnMatch) {
        context.report({
          node,
          messageId: 'noColorLiteral',
          data: { value: fnMatch[0] },
        });
      }
    }

    return {
      Literal(node) {
        if (typeof node.value === 'string') {
          checkString(node, node.value);
        }
      },
      TemplateElement(node) {
        if (node.value && typeof node.value.raw === 'string') {
          checkString(node, node.value.raw);
        }
      },
    };
  },
};

const customColorPlugin = {
  rules: {
    'no-color-literals': noColorLiteralsRule,
  },
};

export default tseslint.config(
  { ignores: ['dist', 'node_modules', 'src/test/tokens.test.ts'] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      'react-hooks': reactHooks,
      'tokens': customColorPlugin,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'tokens/no-color-literals': 'error',
    },
  }
);