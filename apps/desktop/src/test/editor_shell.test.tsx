import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { EditorShell } from '../components/EditorShell';

describe('EditorShell Component (LC-028 / SPEC-01 Frameless Chrome)', () => {
  it('renders studio titlebar with brand, finding pill, and frameless window controls', () => {
    const handleHistory = vi.fn();
    const handleSettings = vi.fn();

    render(
      <EditorShell
        activeTab="findings"
        onTabChange={vi.fn()}
        onOpenHistory={handleHistory}
        onOpenSettings={handleSettings}
        activeFindingTitle="2026-08-24_19-45.snapx"
      >
        <div>Studio Viewport Child</div>
      </EditorShell>
    );

    const titlebar = screen.getByTestId('studio-titlebar');
    expect(titlebar).toBeInTheDocument();
    expect(screen.getByText('Snapdown Studio')).toBeInTheDocument();
    expect(screen.getByTestId('titlebar-finding-pill')).toHaveTextContent('2026-08-24_19-45.snapx');

    // Action buttons & Frameless Controls
    expect(screen.getByTestId('titlebar-history-btn')).toBeInTheDocument();
    expect(screen.getByTestId('titlebar-settings-btn')).toBeInTheDocument();
    expect(screen.getByTestId('win-minimize-btn')).toBeInTheDocument();
    expect(screen.getByTestId('win-maximize-btn')).toBeInTheDocument();
    expect(screen.getByTestId('win-close-btn')).toBeInTheDocument();

    fireEvent.click(screen.getByTestId('titlebar-history-btn'));
    expect(handleHistory).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByTestId('titlebar-settings-btn'));
    expect(handleSettings).toHaveBeenCalledTimes(1);
  });

  it('renders children within the main studio viewport without crashing', () => {
    render(
      <EditorShell activeTab="findings" onTabChange={vi.fn()}>
        <div data-testid="custom-studio-content">Workspace Artboard</div>
      </EditorShell>
    );

    expect(screen.getByTestId('custom-studio-content')).toHaveTextContent('Workspace Artboard');
  });

  it('provides navigation tab switching callbacks', () => {
    const handleTabChange = vi.fn();
    render(
      <EditorShell activeTab="findings" onTabChange={handleTabChange}>
        <div>Content</div>
      </EditorShell>
    );

    const rail = screen.getByTestId('navigation-rail');
    expect(rail).toBeInTheDocument();
    const findingsTab = screen.getByRole('tab', { name: /findings/i, hidden: true });
    expect(findingsTab).toHaveClass('nav-rail-item');
    fireEvent.click(findingsTab);
    expect(handleTabChange).toHaveBeenCalledWith('findings');
  });
});
