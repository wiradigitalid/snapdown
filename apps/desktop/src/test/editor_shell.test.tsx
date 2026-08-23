import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { EditorShell } from '../components/EditorShell';

describe('EditorShell Component (LC-028)', () => {
  it('renders 200px wide navigation rail with branding', () => {
    const handleTabChange = vi.fn();
    render(
      <EditorShell activeTab="settings" onTabChange={handleTabChange}>
        <div>Content</div>
      </EditorShell>
    );

    const rail = screen.getByTestId('navigation-rail');
    expect(rail).toBeInTheDocument();
    expect(rail).toHaveStyle({ width: '200px' });
    expect(screen.getByText('Snapdown')).toBeInTheDocument();
  });

  it('lists all four primary surfaces (Findings, Bundles, Agent Access, Settings)', () => {
    render(
      <EditorShell activeTab="findings" onTabChange={vi.fn()}>
        <div>Content</div>
      </EditorShell>
    );

    expect(screen.getByRole('tab', { name: /findings/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /bundles/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /agent access/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /settings/i })).toBeInTheDocument();
  });

  it('highlights the active tab with multiple visual cues (background and left edge bar) (NFR-16)', () => {
    render(
      <EditorShell activeTab="bundles" onTabChange={vi.fn()}>
        <div>Content</div>
      </EditorShell>
    );

    const bundlesTab = screen.getByRole('tab', { name: /bundles/i });
    const settingsTab = screen.getByRole('tab', { name: /settings/i });

    expect(bundlesTab).toHaveAttribute('aria-selected', 'true');
    expect(settingsTab).toHaveAttribute('aria-selected', 'false');

    // Multi-signal: background fill and borderLeft edge bar
    expect(bundlesTab).toHaveStyle({
      backgroundColor: 'var(--color-accent)',
    });
    expect(bundlesTab.style.borderLeftWidth).toBe('4px');
    expect(bundlesTab.style.borderLeftStyle).toBe('solid');
    expect(bundlesTab.style.borderLeftColor).toBe('var(--color-accent-text)');
    expect(settingsTab.style.borderLeftColor).toBe('transparent');
  });

  it('triggers onTabChange when navigation tab is clicked', () => {
    const handleTabChange = vi.fn();
    render(
      <EditorShell activeTab="findings" onTabChange={handleTabChange}>
        <div>Content</div>
      </EditorShell>
    );

    const settingsTab = screen.getByRole('tab', { name: /settings/i });
    fireEvent.click(settingsTab);

    expect(handleTabChange).toHaveBeenCalledWith('settings');
  });

  it('renders pinned Capture button and triggers onCaptureClick', () => {
    const handleCaptureClick = vi.fn();
    render(
      <EditorShell
        activeTab="findings"
        onTabChange={vi.fn()}
        onCaptureClick={handleCaptureClick}
      >
        <div>Content</div>
      </EditorShell>
    );

    const captureBtn = screen.getByTestId('rail-capture-btn');
    expect(captureBtn).toBeInTheDocument();

    fireEvent.click(captureBtn);
    expect(handleCaptureClick).toHaveBeenCalledTimes(1);
  });

  it('renders children within the main content area without crashing on child state', () => {
    render(
      <EditorShell activeTab="findings" onTabChange={vi.fn()}>
        <div data-testid="custom-child-view">Surface Child</div>
      </EditorShell>
    );

    expect(screen.getByTestId('custom-child-view')).toHaveTextContent('Surface Child');
  });
  it('provides visible focus-visible treatment without suppressing focus outline (NFR-16, EXPERIENCE)', () => {
    render(
      <EditorShell activeTab="findings" onTabChange={vi.fn()} onCaptureClick={vi.fn()}>
        <div>Content</div>
      </EditorShell>
    );

    const findingsTab = screen.getByRole('tab', { name: /findings/i });
    const settingsTab = screen.getByRole('tab', { name: /settings/i });
    const captureBtn = screen.getByTestId('rail-capture-btn');

    // Navigation rail items must carry the nav-rail-item class
    expect(findingsTab).toHaveClass('nav-rail-item');
    expect(settingsTab).toHaveClass('nav-rail-item');
    expect(captureBtn).toHaveClass('rail-capture-btn');

    // Focus must NOT be suppressed by inline outline: 'none'
    expect(findingsTab.style.outline).not.toBe('none');
    expect(settingsTab.style.outline).not.toBe('none');
    expect(captureBtn.style.outline).not.toBe('none');
  });
});
