import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import fs from 'node:fs';
import path from 'node:path';
import { BundlesEditor, BundleDetailDto } from '../components/BundlesEditor';

const mockBundles: BundleDetailDto[] = [
  {
    bundle: {
      id: 'b-1',
      name: 'checkout-pass-1',
      markdown: '# checkout-pass-1\n\n![](./img/f-01.jpg)\n\n1. the CTA drops below the fold',
      markdown_path: 'bundles/checkout-pass-1.md',
      composed_at: '2026-08-12T14:30:00Z',
    },
    items: [
      {
        id: 'item-1',
        bundle_id: 'b-1',
        finding_id: 'f-01',
        position: 1,
        image_path: 'findings/f-01.png',
        note_first_line: 'the CTA drops below the fold',
      },
      {
        id: 'item-2',
        bundle_id: 'b-1',
        finding_id: 'f-02',
        position: 2,
        image_path: 'findings/f-02.png',
        note_first_line: 'spacing on the summary row',
      },
    ],
  },
  {
    bundle: {
      id: 'b-2',
      name: 'nav-review',
      markdown: '# nav-review\n\n- item 1',
      markdown_path: 'bundles/nav-review.md',
      composed_at: '2026-08-04T10:00:00Z',
    },
    items: [
      {
        id: 'item-3',
        bundle_id: 'b-2',
        finding_id: 'f-03',
        position: 1,
        image_path: 'findings/f-03.png',
        note_first_line: 'contrast issue on nav items',
      },
    ],
  },
];

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
  throw new Error(`Unsupported color: ${color}`);
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

describe('BundlesEditor (LC-014 / W6-S8)', () => {
  it('the_preview_is_a_read_only_region_and_not_a_disabled_input', () => {
    render(
      <BundlesEditor
        bundles={mockBundles}
        selectedBundleId="b-1"
        onSelectBundle={vi.fn()}
        onCopyMarkdown={vi.fn()}
      />
    );

    const previewRegion = screen.getByRole('region', { name: 'Markdown Preview' });
    expect(previewRegion).toBeInTheDocument();
    expect(previewRegion.tagName.toLowerCase()).not.toBe('textarea');
    expect(previewRegion.tagName.toLowerCase()).not.toBe('input');
    expect(previewRegion).not.toHaveAttribute('disabled');
    expect(previewRegion).toHaveTextContent('# checkout-pass-1');
    expect(previewRegion).toHaveTextContent('the CTA drops below the fold');

    // Asserts cursor is default and text selectable
    expect(previewRegion).toHaveStyle({ cursor: 'default', userSelect: 'text' });
  });

  it('the_empty_state_offers_no_button_that_only_navigates_away', () => {
    const { container } = render(
      <BundlesEditor
        bundles={[]}
        selectedBundleId={null}
        isLoading={false}
        onSelectBundle={vi.fn()}
        onCopyMarkdown={vi.fn()}
      />
    );

    const emptyState = screen.getByTestId('bundles-empty-state');
    expect(emptyState).toBeInTheDocument();
    expect(screen.getByText('No bundles yet')).toBeInTheDocument();
    expect(
      screen.getByText('Select findings on the Findings tab and choose Compose.')
    ).toBeInTheDocument();

    const buttons = container.querySelectorAll('button');
    expect(buttons).toHaveLength(0);
  });

  it('an_item_whose_image_copy_is_missing_is_flagged_and_the_bundle_still_opens', () => {
    const bundlesWithMissingItem: BundleDetailDto[] = [
      {
        bundle: {
          id: 'b-missing',
          name: 'missing-asset-bundle',
          markdown: '# Bundle With Missing Item\n\n![](./img/gone.png)\n\n1. note',
          markdown_path: 'bundles/missing.md',
          composed_at: '2026-08-20T08:00:00Z',
        },
        items: [
          {
            id: 'item-intact',
            bundle_id: 'b-missing',
            finding_id: 'f-intact',
            position: 1,
            image_path: 'findings/intact.png',
            note_first_line: 'Intact finding',
            is_missing: false,
          },
          {
            id: 'item-missing',
            bundle_id: 'b-missing',
            finding_id: 'f-missing',
            position: 2,
            image_path: 'findings/missing.png',
            note_first_line: 'Missing image finding',
            is_missing: true,
          },
        ],
      },
    ];

    render(
      <BundlesEditor
        bundles={bundlesWithMissingItem}
        selectedBundleId="b-missing"
        onSelectBundle={vi.fn()}
        onCopyMarkdown={vi.fn()}
        onDeleteBundle={vi.fn()}
      />
    );

    expect(screen.getByTestId('item-missing-badge')).toBeInTheDocument();
    expect(screen.getByText('Missing')).toBeInTheDocument();

    const preview = screen.getByTestId('bundle-markdown-preview');
    expect(preview).toHaveTextContent('# Bundle With Missing Item');

    expect(screen.getByTestId('copy-markdown-btn')).toBeInTheDocument();
    expect(screen.getByTestId('copy-markdown-btn')).toBeEnabled();
    expect(screen.getByTestId('delete-bundle-btn')).toBeInTheDocument();
    expect(screen.getByTestId('delete-bundle-btn')).toBeEnabled();
  });

  it('bundles_renders_correctly_in_both_windows_themes', () => {
    const tokensCssPath = path.resolve(process.cwd(), 'src/styles/tokens.css');
    const content = fs.readFileSync(tokensCssPath, 'utf-8');

    expect(content).toContain('--bundle-list-width: 240px');
    expect(content).toContain('--item-list-width: 280px');
    expect(content).toContain('--preview-line-height: 1.55');

    const rootMatch = content.match(/:root\s*\{([\s\S]*?)\}/);
    const darkMatch = content.match(/@media\s*\(prefers-color-scheme:\s*dark\)\s*\{([\s\S]*)\}/);

    expect(rootMatch).toBeTruthy();
    expect(darkMatch).toBeTruthy();

    const lightSunken = rootMatch![1].match(/--color-surface-sunken:\s*([^;]+);/)![1].trim();
    const lightText = rootMatch![1].match(/--color-text:\s*([^;]+);/)![1].trim();

    const darkSunken = darkMatch![1].match(/--color-surface-sunken:\s*([^;]+);/)![1].trim();
    const darkText = darkMatch![1].match(/--color-text:\s*([^;]+);/)![1].trim();

    expect(calculateContrast(lightText, lightSunken)).toBeGreaterThanOrEqual(4.5);
    expect(calculateContrast(darkText, darkSunken)).toBeGreaterThanOrEqual(4.5);
  });

  it('copy_markdown_announces_its_result', () => {
    const handleCopy = vi.fn();
    render(
      <BundlesEditor
        bundles={mockBundles}
        selectedBundleId="b-1"
        onSelectBundle={vi.fn()}
        onCopyMarkdown={handleCopy}
      />
    );

    const copyBtn = screen.getByTestId('copy-markdown-btn');
    expect(copyBtn).toBeInTheDocument();
    fireEvent.click(copyBtn);

    expect(handleCopy).toHaveBeenCalledTimes(1);
    expect(handleCopy).toHaveBeenCalledWith('b-1');
  });

  it('skeleton loading state renders placeholders while maintaining 3-column layout', () => {
    render(
      <BundlesEditor
        bundles={[]}
        selectedBundleId={null}
        isLoading={true}
        onSelectBundle={vi.fn()}
        onCopyMarkdown={vi.fn()}
      />
    );

    expect(screen.getByTestId('bundles-editor')).toBeInTheDocument();
    expect(screen.getByTestId('bundle-list-pane')).toBeInTheDocument();
    expect(screen.getByTestId('bundle-preview-pane')).toBeInTheDocument();
    expect(screen.getByTestId('bundle-items-pane')).toBeInTheDocument();
    expect(screen.getAllByTestId('bundle-skeleton-row').length).toBe(3);
  });

  it('renders nothing-selected prompt when bundles exist but none is selected', () => {
    render(
      <BundlesEditor
        bundles={mockBundles}
        selectedBundleId={null}
        onSelectBundle={vi.fn()}
        onCopyMarkdown={vi.fn()}
      />
    );

    expect(screen.getByTestId('bundle-preview-empty')).toHaveTextContent(
      'Select a bundle to preview content.'
    );
    expect(screen.getByText('No bundle selected')).toBeInTheDocument();
  });

  it('renders centered ErrorState when error is passed and triggers onRetry', () => {
    const handleRetry = vi.fn();
    render(
      <BundlesEditor
        bundles={[]}
        selectedBundleId={null}
        error="Vault database unreadable"
        onSelectBundle={vi.fn()}
        onCopyMarkdown={vi.fn()}
        onRetry={handleRetry}
      />
    );

    expect(screen.getByTestId('bundles-error-state')).toBeInTheDocument();
    expect(screen.getByText('The Library could not be read')).toBeInTheDocument();
    expect(screen.getByText('Vault database unreadable')).toBeInTheDocument();

    const retryBtn = screen.getByText('Retry');
    fireEvent.click(retryBtn);
    expect(handleRetry).toHaveBeenCalledTimes(1);
  });

  it('allows keyboard navigation in bundle list', () => {
    const handleSelect = vi.fn();
    render(
      <BundlesEditor
        bundles={mockBundles}
        selectedBundleId="b-1"
        onSelectBundle={handleSelect}
        onCopyMarkdown={vi.fn()}
      />
    );

    const item2 = screen.getByTestId('bundle-item-b-2');
    fireEvent.keyDown(item2, { key: 'Enter' });
    expect(handleSelect).toHaveBeenCalledWith('b-2');
  });
});
