import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';

describe('Design Tokens Integrity and Parity', () => {
  const tokensCssPath = path.resolve(process.cwd(), 'src/styles/tokens.css');
  const tokensCssContent = fs.readFileSync(tokensCssPath, 'utf-8');

  it('defines all required design tokens in :root (light theme)', () => {
    const requiredTokens = [
      '--color-bg',
      '--color-surface',
      '--color-surface-raised',
      '--color-surface-sunken',
      '--color-text',
      '--color-text-muted',
      '--color-border',
      '--color-border-strong',
      '--color-accent',
      '--color-accent-text',
      '--color-danger',
      '--color-danger-text',
      '--color-success-bg',
      '--color-success-text',
      '--color-warning-bg',
      '--color-warning-text',
      '--color-info-bg',
      '--color-info-text',
      '--color-neutral-bg',
      '--color-neutral-text',
      '--color-marker',
      '--color-marker-text',
      '--color-marker-ring',
      '--color-overlay-scrim',
      '--color-overlay-ring',
      '--canvas-checker',
      '--space-0',
      '--space-1',
      '--space-2',
      '--space-3',
      '--space-4',
      '--space-5',
      '--space-6',
      '--radius-sm',
      '--radius-md',
      '--radius-full',
      '--font-ui',
      '--font-mono',
      '--text-xs',
      '--text-sm',
      '--text-base',
      '--text-lg',
      '--text-xl',
      '--shadow-raised',
      '--z-overlay',
      '--z-toast',
      '--z-modal',
    ];

    for (const token of requiredTokens) {
      expect(tokensCssContent).toContain(token);
    }
  });

  it('defines dark theme overrides under @media (prefers-color-scheme: dark)', () => {
    expect(tokensCssContent).toContain('@media (prefers-color-scheme: dark)');
    const darkBlockMatch = tokensCssContent.match(/@media\s*\(prefers-color-scheme:\s*dark\)\s*\{([\s\S]*)\}/);
    expect(darkBlockMatch).toBeTruthy();
    const darkContent = darkBlockMatch![1];

    const requiredDarkTokens = [
      '--color-bg',
      '--color-surface',
      '--color-surface-raised',
      '--color-surface-sunken',
      '--color-text',
      '--color-text-muted',
      '--color-border',
      '--color-border-strong',
      '--color-accent',
      '--color-accent-text',
      '--color-danger',
      '--color-danger-text',
      '--color-success-bg',
      '--color-success-text',
      '--color-warning-bg',
      '--color-warning-text',
      '--color-info-bg',
      '--color-info-text',
      '--color-neutral-bg',
      '--color-neutral-text',
      '--color-marker',
      '--color-marker-text',
      '--color-marker-ring',
      '--color-overlay-scrim',
      '--color-overlay-ring',
      '--canvas-checker',
    ];

    for (const token of requiredDarkTokens) {
      expect(darkContent).toContain(token);
    }
  });

  it('ensures theme-invariant tokens retain identical values in both themes', () => {
    // Marker tokens must be invariant: amber, black text, white ring
    const markerMatch = tokensCssContent.match(/--color-marker:\s*([^;]+);/g);
    expect(markerMatch).toHaveLength(2);
    expect(markerMatch![0]).toBe(markerMatch![1]);

    const markerTextMatch = tokensCssContent.match(/--color-marker-text:\s*([^;]+);/g);
    expect(markerTextMatch).toHaveLength(2);
    expect(markerTextMatch![0]).toBe(markerTextMatch![1]);

    const markerRingMatch = tokensCssContent.match(/--color-marker-ring:\s*([^;]+);/g);
    expect(markerRingMatch).toHaveLength(2);
    expect(markerRingMatch![0]).toBe(markerRingMatch![1]);
  });

  it('ensures zero color literals exist in source trees outside tokens.css', () => {
    const HEX_COLOR_REGEX = /#(?:[0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b/g;

    function findFiles(dir: string, extFilter: (f: string) => boolean): string[] {
      const results: string[] = [];
      if (!fs.existsSync(dir)) return results;
      const list = fs.readdirSync(dir);
      for (const file of list) {
        const fullPath = path.join(dir, file);
        const stat = fs.statSync(fullPath);
        if (stat && stat.isDirectory()) {
          if (file !== 'node_modules' && file !== 'dist') {
            results.push(...findFiles(fullPath, extFilter));
          }
        } else if (extFilter(fullPath)) {
          results.push(fullPath);
        }
      }
      return results;
    }

    const cwd = process.cwd();
    const webUiSrc = path.resolve(cwd, 'src');
    const desktopSrc = path.resolve(cwd, '../desktop/src');

    const sourceFiles = [
      ...findFiles(webUiSrc, (f) => /\.(tsx?|css)$/.test(f)),
      ...findFiles(desktopSrc, (f) => /\.(tsx?|css)$/.test(f)),
    ].filter(
      (f) =>
        !f.endsWith('tokens.css') &&
        !f.endsWith('tokens.test.ts')
    );

    expect(sourceFiles.length).toBeGreaterThan(5);

    const violations: { file: string; match: string }[] = [];

    for (const filePath of sourceFiles) {
      const content = fs.readFileSync(filePath, 'utf-8');
      const matches = content.match(HEX_COLOR_REGEX);
      if (matches) {
        for (const match of matches) {
          violations.push({ file: filePath, match });
        }
      }
    }

    expect(violations).toEqual([]);
  });
});