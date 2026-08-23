import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';

function parseTokenDeclarations(block: string): Record<string, string> {
  const tokens: Record<string, string> = {};
  const regex = /(--[\w-]+)\s*:\s*([^;]+);/g;
  let match: RegExpExecArray | null;
  while ((match = regex.exec(block)) !== null) {
    tokens[match[1].trim()] = match[2].trim();
  }
  return tokens;
}

function parseTokensFile(filePath: string): {
  lightTokens: Record<string, string>;
  darkTokens: Record<string, string>;
} {
  const content = fs.readFileSync(filePath, 'utf-8');

  const rootBlockMatch = content.match(/:root\s*\{([\s\S]*?)\}/);
  if (!rootBlockMatch) {
    throw new Error(`Failed to find :root block in ${filePath}`);
  }
  const lightTokens = parseTokenDeclarations(rootBlockMatch[1]);

  const darkMediaMatch = content.match(/@media\s*\(prefers-color-scheme:\s*dark\)\s*\{([\s\S]*)\}/);
  if (!darkMediaMatch) {
    throw new Error(`Failed to find @media (prefers-color-scheme: dark) block in ${filePath}`);
  }
  const darkTokens = parseTokenDeclarations(darkMediaMatch[1]);

  return { lightTokens, darkTokens };
}

function parseColorToRgb(color: string): [number, number, number] {
  const trimmed = color.trim();
  if (trimmed.startsWith('#')) {
    let hex = trimmed.replace(/^#/, '');
    if (hex.length === 3) {
      hex = hex.split('').map((c) => c + c).join('');
    }
    const num = parseInt(hex, 16);
    return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
  }
  const rgbMatch = trimmed.match(/^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/);
  if (rgbMatch) {
    return [parseInt(rgbMatch[1], 10), parseInt(rgbMatch[2], 10), parseInt(rgbMatch[3], 10)];
  }
  throw new Error(`Unsupported color format: ${color}`);
}

function relativeLuminance(r: number, g: number, b: number): number {
  const [rs, gs, bs] = [r, g, b].map((c) => {
    const s = c / 255;
    return s <= 0.04045 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

function calculateContrast(fgColor: string, bgColor: string): number {
  const [r1, g1, b1] = parseColorToRgb(fgColor);
  const [r2, g2, b2] = parseColorToRgb(bgColor);
  const l1 = relativeLuminance(r1, g1, b1);
  const l2 = relativeLuminance(r2, g2, b2);
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
}

const tokensCssPath = fs.existsSync(path.resolve(process.cwd(), 'src/styles/tokens.css'))
  ? path.resolve(process.cwd(), 'src/styles/tokens.css')
  : path.resolve(process.cwd(), 'web/ui/src/styles/tokens.css');

const { lightTokens, darkTokens } = parseTokensFile(tokensCssPath);

describe('Design Tokens Contrast Verification (WCAG AA)', () => {
  it('proves the contrast parse is live from tokens.css file', () => {
    expect(lightTokens['--color-text']).toBeDefined();
    expect(lightTokens['--color-bg']).toBeDefined();
    expect(darkTokens['--color-text']).toBeDefined();
    expect(darkTokens['--color-bg']).toBeDefined();

    const liveLightRatio = calculateContrast(lightTokens['--color-text'], lightTokens['--color-bg']);
    const liveDarkRatio = calculateContrast(darkTokens['--color-text'], darkTokens['--color-bg']);

    expect(typeof liveLightRatio).toBe('number');
    expect(liveLightRatio).toBeGreaterThanOrEqual(4.5);
    expect(typeof liveDarkRatio).toBe('number');
    expect(liveDarkRatio).toBeGreaterThanOrEqual(4.5);
  });

  describe('Light Theme Contrast Ratios', () => {
    it('satisfies WCAG AA (>= 4.5:1) for body text against background and surfaces', () => {
      expect(calculateContrast(lightTokens['--color-text'], lightTokens['--color-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens['--color-text'], lightTokens['--color-surface'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens['--color-text'], lightTokens['--color-surface-sunken'])).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA (>= 4.5:1) for muted text against surfaces', () => {
      expect(calculateContrast(lightTokens['--color-text-muted'], lightTokens['--color-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens['--color-text-muted'], lightTokens['--color-surface'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens['--color-text-muted'], lightTokens['--color-surface-sunken'])).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA for primary action and danger buttons', () => {
      expect(calculateContrast(lightTokens['--color-accent-text'], lightTokens['--color-accent'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens['--color-danger-text'], lightTokens['--color-danger'])).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA (>= 4.5:1) for all semantic meaning token pairs against their own background', () => {
      expect(calculateContrast(lightTokens['--color-success-text'], lightTokens['--color-success-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens['--color-warning-text'], lightTokens['--color-warning-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens['--color-info-text'], lightTokens['--color-info-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens['--color-neutral-text'], lightTokens['--color-neutral-bg'])).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA for theme-invariant marker badge', () => {
      expect(calculateContrast(lightTokens['--color-marker-text'], lightTokens['--color-marker'])).toBeGreaterThanOrEqual(4.5);
    });
  });

  describe('Dark Theme Contrast Ratios', () => {
    it('satisfies WCAG AA (>= 4.5:1) for body text against background and surfaces', () => {
      expect(calculateContrast(darkTokens['--color-text'], darkTokens['--color-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens['--color-text'], darkTokens['--color-surface'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens['--color-text'], darkTokens['--color-surface-sunken'])).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA (>= 4.5:1) for muted text against surfaces', () => {
      expect(calculateContrast(darkTokens['--color-text-muted'], darkTokens['--color-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens['--color-text-muted'], darkTokens['--color-surface'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens['--color-text-muted'], darkTokens['--color-surface-sunken'])).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA for primary action and danger buttons', () => {
      expect(calculateContrast(darkTokens['--color-accent-text'], darkTokens['--color-accent'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens['--color-danger-text'], darkTokens['--color-danger'])).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA (>= 4.5:1) for all semantic meaning token pairs against their own background', () => {
      expect(calculateContrast(darkTokens['--color-success-text'], darkTokens['--color-success-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens['--color-warning-text'], darkTokens['--color-warning-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens['--color-info-text'], darkTokens['--color-info-bg'])).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens['--color-neutral-text'], darkTokens['--color-neutral-bg'])).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA for theme-invariant marker badge', () => {
      expect(calculateContrast(darkTokens['--color-marker-text'], darkTokens['--color-marker'])).toBeGreaterThanOrEqual(4.5);
    });
  });
});