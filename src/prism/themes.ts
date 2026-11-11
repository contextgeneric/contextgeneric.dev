import type {PrismTheme} from 'prism-react-renderer';

// The code colours for the site's two themes, made for the palette in src/css/custom.css. The plain
// background matches the `--cgp-code-bg` token of each theme. No token uses the brand red, so code
// never looks like a link or an error, and each colour keeps WCAG AA contrast against the background,
// and against a highlighted line's tint.

export const lightTheme: PrismTheme = {
  plain: {
    color: '#261d1b',
    backgroundColor: '#f4ece5',
  },
  styles: [
    {
      types: ['comment', 'prolog', 'doctype', 'cdata', 'doc-comment'],
      style: {color: '#6e5f58', fontStyle: 'italic'},
    },
    {
      types: ['keyword', 'selector', 'important', 'atrule', 'rule'],
      style: {color: '#7a2f6b'},
    },
    {
      types: ['attribute', 'attr-name', 'macro', 'property-access', 'lifetime-annotation', 'symbol'],
      style: {color: '#7d5200'},
    },
    {
      types: ['function', 'function-definition', 'method'],
      style: {color: '#1d6363'},
    },
    {
      types: ['string', 'char', 'attr-value', 'regex', 'url', 'inserted'],
      style: {color: '#3b6527'},
    },
    {
      types: ['number', 'boolean', 'constant'],
      style: {color: '#994412'},
    },
    {
      types: ['class-name', 'builtin', 'type-definition', 'namespace'],
      style: {color: '#2f5577'},
    },
    {
      types: ['punctuation', 'operator', 'closure-punctuation'],
      style: {color: '#594b47'},
    },
    {
      types: ['deleted'],
      style: {color: '#a8322a'},
    },
    {
      types: ['bold', 'important'],
      style: {fontWeight: 'bold'},
    },
    {
      types: ['italic'],
      style: {fontStyle: 'italic'},
    },
  ],
};

export const darkTheme: PrismTheme = {
  plain: {
    color: '#ece3df',
    backgroundColor: '#211918',
  },
  styles: [
    {
      types: ['comment', 'prolog', 'doctype', 'cdata', 'doc-comment'],
      style: {color: '#ae9f99', fontStyle: 'italic'},
    },
    {
      types: ['keyword', 'selector', 'important', 'atrule', 'rule'],
      style: {color: '#e3a0d6'},
    },
    {
      types: ['attribute', 'attr-name', 'macro', 'property-access', 'lifetime-annotation', 'symbol'],
      style: {color: '#e8bd66'},
    },
    {
      types: ['function', 'function-definition', 'method'],
      style: {color: '#82cfc6'},
    },
    {
      types: ['string', 'char', 'attr-value', 'regex', 'url', 'inserted'],
      style: {color: '#a9cf8c'},
    },
    {
      types: ['number', 'boolean', 'constant'],
      style: {color: '#f2a66e'},
    },
    {
      types: ['class-name', 'builtin', 'type-definition', 'namespace'],
      style: {color: '#9cbde3'},
    },
    {
      types: ['punctuation', 'operator', 'closure-punctuation'],
      style: {color: '#c4b5b0'},
    },
    {
      types: ['deleted'],
      style: {color: '#f08a80'},
    },
    {
      types: ['bold', 'important'],
      style: {fontWeight: 'bold'},
    },
    {
      types: ['italic'],
      style: {fontStyle: 'italic'},
    },
  ],
};
