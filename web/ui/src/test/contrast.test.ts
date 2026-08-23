import { describe, it, expect } from 'vitest';

function hexToRgb(hex: string): [number, number, number] {
  let cleaned = hex.replace(/^#/, '');
  if (cleaned.length === 3) {
    cleaned = cleaned.split('').map((c) => c + c).join('');
  }
  const num = parseInt(cleaned, 16);
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
}

function relativeLuminance(r: number, g: number, b: number): number {
  const [rs, gs, bs] = [r, g, b].map((c) => {
    const s = c / 255;
    return s <= 0.04045 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

function calculateContrast(fgHex: string, bgHex: string): number {
  const [r1, g1, b1] = hexToRgb(fgHex);
  const [r2, g2, b2] = hexToRgb(bgHex);
  const l1 = relativeLuminance(r1, g1, b1);
  const l2 = relativeLuminance(r2, g2, b2);
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
}

describe('Design Tokens Contrast Verification (WCAG AA)', () => {
  describe('Light Theme Contrast Ratios', () => {
    const lightTokens = {
      bg: '#f8fafc',
      surface: '#ffffff',
      surfaceRaised: '#ffffff',
      surfaceSunken: '#f1f5f9',
      text: '#0f172a',
      textMuted: '#475569',
      accent: '#2563eb',
      accentText: '#ffffff',
      danger: '#dc2626',
      dangerText: '#ffffff',
      successBg: '#dcfce7',
      successText: '#166534',
      warningBg: '#fef3c7',
      warningText: '#854d0e',
      infoBg: '#eff6ff',
      infoText: '#1e40af',
      neutralBg: '#f1f5f9',
      neutralText: '#334155',
      marker: '#f59e0b',
      markerText: '#000000',
    };

    it('satisfies WCAG AA (>= 4.5:1) for body text against background and surfaces', () => {
      expect(calculateContrast(lightTokens.text, lightTokens.bg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens.text, lightTokens.surface)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens.text, lightTokens.surfaceSunken)).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA (>= 4.5:1) for muted text against surfaces', () => {
      expect(calculateContrast(lightTokens.textMuted, lightTokens.bg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens.textMuted, lightTokens.surface)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens.textMuted, lightTokens.surfaceSunken)).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA for primary action and danger buttons', () => {
      expect(calculateContrast(lightTokens.accentText, lightTokens.accent)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens.dangerText, lightTokens.danger)).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA (>= 4.5:1) for all semantic meaning token pairs against their own background', () => {
      expect(calculateContrast(lightTokens.successText, lightTokens.successBg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens.warningText, lightTokens.warningBg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens.infoText, lightTokens.infoBg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(lightTokens.neutralText, lightTokens.neutralBg)).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA for theme-invariant marker badge', () => {
      expect(calculateContrast(lightTokens.markerText, lightTokens.marker)).toBeGreaterThanOrEqual(4.5);
    });
  });

  describe('Dark Theme Contrast Ratios', () => {
    const darkTokens = {
      bg: '#090d16',
      surface: '#131b2e',
      surfaceRaised: '#1e293b',
      surfaceSunken: '#0f172a',
      text: '#f8fafc',
      textMuted: '#94a3b8',
      accent: '#2563eb',
      accentText: '#ffffff',
      danger: '#dc2626',
      dangerText: '#ffffff',
      successBg: '#052e16',
      successText: '#4ade80',
      warningBg: '#451a03',
      warningText: '#fef08a',
      infoBg: '#172554',
      infoText: '#93c5fd',
      neutralBg: '#1e293b',
      neutralText: '#e2e8f0',
      marker: '#f59e0b',
      markerText: '#000000',
    };

    it('satisfies WCAG AA (>= 4.5:1) for body text against background and surfaces', () => {
      expect(calculateContrast(darkTokens.text, darkTokens.bg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens.text, darkTokens.surface)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens.text, darkTokens.surfaceSunken)).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA (>= 4.5:1) for muted text against surfaces', () => {
      expect(calculateContrast(darkTokens.textMuted, darkTokens.bg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens.textMuted, darkTokens.surface)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens.textMuted, darkTokens.surfaceSunken)).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA for primary action and danger buttons', () => {
      expect(calculateContrast(darkTokens.accentText, darkTokens.accent)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens.dangerText, darkTokens.danger)).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA (>= 4.5:1) for all semantic meaning token pairs against their own background', () => {
      expect(calculateContrast(darkTokens.successText, darkTokens.successBg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens.warningText, darkTokens.warningBg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens.infoText, darkTokens.infoBg)).toBeGreaterThanOrEqual(4.5);
      expect(calculateContrast(darkTokens.neutralText, darkTokens.neutralBg)).toBeGreaterThanOrEqual(4.5);
    });

    it('satisfies WCAG AA for theme-invariant marker badge', () => {
      expect(calculateContrast(darkTokens.markerText, darkTokens.marker)).toBeGreaterThanOrEqual(4.5);
    });
  });
});
