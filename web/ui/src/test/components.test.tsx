import fs from 'node:fs';
import path from 'node:path';
import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import {
  Button,
  TextField,
  TextArea,
  Checkbox,
  Toast,
  Modal,
  ConfirmDialog,
  MarkerBadge,
  EmptyState,
  ErrorState,
  Badge,
  Toggle,
  SegmentedControl,
  HotkeyChip,
} from '../index';

describe('web/ui components suite', () => {
  describe('Button', () => {
    it('renders with primary variant and handles click', () => {
      const handleClick = vi.fn();
      render(<Button variant="primary" onClick={handleClick}>Save</Button>);
      const btn = screen.getByRole('button', { name: 'Save' });
      expect(btn).toHaveClass('btn-primary');
      fireEvent.click(btn);
      expect(handleClick).toHaveBeenCalledTimes(1);
    });

    it('supports disabled and loading states', () => {
      const { rerender } = render(<Button disabled>Disabled</Button>);
      const btn = screen.getByRole('button', { name: 'Disabled' });
      expect(btn).toBeDisabled();

      rerender(<Button loading>Loading</Button>);
      expect(screen.getByRole('button', { name: '... Loading' })).toBeDisabled();
      expect(screen.getByRole('button', { name: '... Loading' })).toHaveClass('loading');
    });
  });

  describe('Badge', () => {
    it('renders different variants with semantic meaning classes', () => {
      const { rerender } = render(<Badge variant="success">Active</Badge>);
      expect(screen.getByText('Active')).toHaveClass('badge-success');

      rerender(<Badge variant="warning">Warning</Badge>);
      expect(screen.getByText('Warning')).toHaveClass('badge-warning');

      rerender(<Badge variant="info">Info</Badge>);
      expect(screen.getByText('Info')).toHaveClass('badge-info');

      rerender(<Badge variant="danger">Danger</Badge>);
      expect(screen.getByText('Danger')).toHaveClass('badge-danger');

      rerender(<Badge variant="neutral">Neutral</Badge>);
      expect(screen.getByText('Neutral')).toHaveClass('badge-neutral');
    });
  });

  describe('Toggle', () => {
    it('renders on and off states and triggers onChange', () => {
      const handleChange = vi.fn();
      const { rerender } = render(<Toggle checked={false} onChange={handleChange} aria-label="Dark Mode" />);
      const toggle = screen.getByRole('switch', { name: 'Dark Mode' });
      expect(toggle).toHaveAttribute('aria-checked', 'false');
      expect(toggle).toHaveAttribute('data-state', 'off');

      fireEvent.click(toggle);
      expect(handleChange).toHaveBeenCalledWith(true);

      rerender(<Toggle checked={true} onChange={handleChange} aria-label="Dark Mode" />);
      expect(toggle).toHaveAttribute('aria-checked', 'true');
      expect(toggle).toHaveAttribute('data-state', 'on');
    });

    it('renders distinct indeterminate state (FR-18, load-bearing for async OS reads)', () => {
      render(<Toggle indeterminate checked={false} aria-label="Startup Status" />);
      const toggle = screen.getByRole('switch', { name: 'Startup Status' });
      expect(toggle).toHaveAttribute('aria-checked', 'mixed');
      expect(toggle).toHaveAttribute('data-state', 'indeterminate');
    });

    it('supports keyboard Space and Enter keys', () => {
      const handleChange = vi.fn();
      render(<Toggle checked={false} onChange={handleChange} aria-label="Toggle Option" />);
      const toggle = screen.getByRole('switch', { name: 'Toggle Option' });

      fireEvent.keyDown(toggle, { key: ' ' });
      expect(handleChange).toHaveBeenCalledWith(true);

      fireEvent.keyDown(toggle, { key: 'Enter' });
      expect(handleChange).toHaveBeenCalledWith(true);
    });

    it('ignores clicks and key presses when disabled', () => {
      const handleChange = vi.fn();
      render(<Toggle disabled checked={false} onChange={handleChange} aria-label="Disabled Toggle" />);
      const toggle = screen.getByRole('switch', { name: 'Disabled Toggle' });
      expect(toggle).toBeDisabled();

      fireEvent.click(toggle);
      fireEvent.keyDown(toggle, { key: ' ' });
      expect(handleChange).not.toHaveBeenCalled();
    });
  });

  describe('SegmentedControl', () => {
    const options = [
      { value: 'tab1', label: 'Tab 1' },
      { value: 'tab2', label: 'Tab 2' },
      { value: 'tab3', label: 'Tab 3', disabled: true },
    ];

    it('renders options and updates value on click', () => {
      const handleChange = vi.fn();
      render(
        <SegmentedControl
          options={options}
          value="tab1"
          onChange={handleChange}
          aria-label="View Switcher"
        />
      );

      const tab2 = screen.getByRole('radio', { name: 'Tab 2' });
      expect(tab2).toHaveAttribute('aria-selected', 'false');

      fireEvent.click(tab2);
      expect(handleChange).toHaveBeenCalledWith('tab2');
    });

    it('supports arrow key navigation (ArrowRight / ArrowLeft)', () => {
      const handleChange = vi.fn();
      render(
        <SegmentedControl
          options={options}
          value="tab1"
          onChange={handleChange}
          aria-label="View Switcher"
        />
      );

      const tab1 = screen.getByRole('radio', { name: 'Tab 1' });
      fireEvent.keyDown(tab1, { key: 'ArrowRight' });
      expect(handleChange).toHaveBeenCalledWith('tab2');
    });
  });

  describe('HotkeyChip', () => {
    it('renders bound shortcut at rest', () => {
      render(<HotkeyChip shortcut="Ctrl+Alt+S" />);
      expect(screen.getByText('Ctrl+Alt+S')).toBeInTheDocument();
      expect(screen.getByRole('button')).toHaveAttribute('data-state', 'bound');
    });

    it('enters listening state on click and captures key combo', () => {
      const handleRecord = vi.fn();
      render(<HotkeyChip shortcut="Ctrl+S" onRecord={handleRecord} />);
      const chip = screen.getByRole('button');

      fireEvent.click(chip);
      expect(screen.getByText('Press shortcut keys (ESC to cancel)...')).toBeInTheDocument();

      fireEvent.keyDown(chip, { key: 'K', ctrlKey: true });
      expect(handleRecord).toHaveBeenCalledWith('CommandOrControl+K');
    });

    it('cancels listening on Escape key', () => {
      const handleCancel = vi.fn();
      render(<HotkeyChip shortcut="Ctrl+S" onCancel={handleCancel} />);
      const chip = screen.getByRole('button');

      fireEvent.click(chip);
      fireEvent.keyDown(chip, { key: 'Escape' });
      expect(handleCancel).toHaveBeenCalledTimes(1);
    });
  });

  describe('EmptyState', () => {
    it('renders illustration, heading, description, and single action button', () => {
      const handleAction = vi.fn();
      render(
        <EmptyState
          illustration={<span data-testid="test-illustration">Icon</span>}
          heading="No Findings Yet"
          description="Use your hotkey to capture your first finding."
          actionLabel="Open Settings"
          onAction={handleAction}
        />
      );

      expect(screen.getByTestId('test-illustration')).toBeInTheDocument();
      expect(screen.getByText('No Findings Yet')).toBeInTheDocument();
      expect(screen.getByText('Use your hotkey to capture your first finding.')).toBeInTheDocument();
      const actionBtn = screen.getByRole('button', { name: 'Open Settings' });
      fireEvent.click(actionBtn);
      expect(handleAction).toHaveBeenCalledTimes(1);
    });
  });

  describe('ErrorState', () => {
    it('renders title, failure message, and actionable retry button', () => {
      const handleRetry = vi.fn();
      render(
        <ErrorState
          title="Capture Failed"
          message="DirectX capture device was disconnected."
          actionLabel="Retry Capture"
          onAction={handleRetry}
        />
      );

      expect(screen.getByText('Capture Failed')).toBeInTheDocument();
      expect(screen.getByText('DirectX capture device was disconnected.')).toBeInTheDocument();
      const retryBtn = screen.getByRole('button', { name: 'Retry Capture' });
      fireEvent.click(retryBtn);
      expect(handleRetry).toHaveBeenCalledTimes(1);
    });
  });

  describe('TextField', () => {
    it('renders label with htmlFor linking to input id', () => {
      render(
        <TextField
          id="vault-path-input"
          label="Vault Path"
          value="test-path"
          readOnly
        />
      );
      const label = screen.getByText('Vault Path');
      expect(label).toHaveAttribute('for', 'vault-path-input');
      const input = screen.getByRole('textbox', { name: 'Vault Path' });
      expect(input).toHaveAttribute('id', 'vault-path-input');
    });

    it('generates fallback id linking label htmlFor to input id', () => {
      render(
        <TextField
          label="Vault Path"
          value="test-path"
          readOnly
        />
      );
      const label = screen.getByText('Vault Path');
      const htmlFor = label.getAttribute('for');
      expect(htmlFor).toBeTruthy();
      const input = screen.getByRole('textbox', { name: 'Vault Path' });
      expect(input.id).toBe(htmlFor);
    });

    it('renders label and character count', () => {
      render(
        <TextField
          label="Vault Path"
          value="test-path"
          showCharCount
          maxLength={50}
          readOnly
        />
      );
      expect(screen.getByText('Vault Path')).toBeInTheDocument();
      expect(screen.getByText('9/50')).toBeInTheDocument();
    });

    it('renders invalid state and error message', () => {
      render(
        <TextField
          invalid
          errorMessage="Invalid path entered"
          defaultValue="invalid"
        />
      );
      expect(screen.getByText('Invalid path entered')).toBeInTheDocument();
      const input = screen.getByDisplayValue('invalid');
      expect(input).toHaveClass('invalid');
    });
  });

  describe('TextArea', () => {
    it('renders label and handles autoGrow', () => {
      render(<TextArea label="Note Body" value="Hello World" readOnly autoGrow />);
      expect(screen.getByText('Note Body')).toBeInTheDocument();
      const textarea = screen.getByDisplayValue('Hello World');
      expect(textarea).toHaveClass('text-area-input');
    });

    it('renders invalid state and error message', () => {
      render(<TextArea invalid errorMessage="Content too long" defaultValue="some long text" />);
      expect(screen.getByText('Content too long')).toBeInTheDocument();
      const textarea = screen.getByDisplayValue('some long text');
      expect(textarea).toHaveClass('invalid');
    });
  });

  describe('Checkbox', () => {
    it('renders label and indeterminate state', () => {
      const { container } = render(<Checkbox label="Select All" indeterminate checked={false} onChange={() => {}} />);
      expect(screen.getByText('Select All')).toBeInTheDocument();
      const input = container.querySelector('input[type="checkbox"]') as HTMLInputElement;
      expect(input.indeterminate).toBe(true);
    });

    it('supports checked and disabled state', () => {
      render(<Checkbox label="Enabled" checked disabled onChange={() => {}} />);
      const checkbox = screen.getByRole('checkbox', { name: 'Enabled' });
      expect(checkbox).toBeChecked();
      expect(checkbox).toBeDisabled();
    });
  });

  describe('Toast', () => {
    it('renders message and auto-dismisses', () => {
      vi.useFakeTimers();
      const handleDismiss = vi.fn();
      render(<Toast message="Captured successfully" durationMs={2000} onDismiss={handleDismiss} />);
      expect(screen.getByText('Captured successfully')).toBeInTheDocument();

      vi.advanceTimersByTime(2000);
      expect(handleDismiss).toHaveBeenCalledTimes(1);
      vi.useRealTimers();
    });

    it('renders reachable action without tabIndex -1', () => {
      const handleAction = vi.fn();
      render(
        <Toast
          message="Saved"
          actionLabel="Undo"
          onAction={handleAction}
          onDismiss={() => {}}
        />
      );
      const btn = screen.getByRole('button', { name: 'Undo' });
      expect(btn).toBeInTheDocument();
      expect(btn).not.toHaveAttribute('tabIndex', '-1');
      fireEvent.click(btn);
      expect(handleAction).toHaveBeenCalledTimes(1);
    });
  });

  describe('Modal', () => {
    it('unmounts when closed via Escape or scrim click from stateful parent', () => {
      const TestParent = () => {
        const [open, setOpen] = React.useState(true);
        return (
          <div>
            <Modal isOpen={open} onClose={() => setOpen(false)} title="Test Modal">
              <button>Inside Button</button>
            </Modal>
          </div>
        );
      };

      const { unmount } = render(<TestParent />);
      expect(screen.getByRole('dialog')).toBeInTheDocument();

      fireEvent.keyDown(window, { key: 'Escape' });
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      unmount();
    });

    it('renders title, content, and traps tab focus', () => {
      const handleClose = vi.fn();
      render(
        <Modal isOpen={true} onClose={handleClose} title="Test Modal">
          <button>First Button</button>
          <button>Second Button</button>
        </Modal>
      );

      expect(screen.getByText('Test Modal')).toBeInTheDocument();
      const dialog = screen.getByRole('dialog');
      expect(dialog).toBeInTheDocument();

      const firstBtn = screen.getByRole('button', { name: 'First Button' });
      const secondBtn = screen.getByRole('button', { name: 'Second Button' });

      // Tab on last element wraps to first
      secondBtn.focus();
      fireEvent.keyDown(window, { key: 'Tab' });
      expect(document.activeElement).toBe(firstBtn);

      // Shift+Tab on first element wraps to last
      firstBtn.focus();
      fireEvent.keyDown(window, { key: 'Tab', shiftKey: true });
      expect(document.activeElement).toBe(secondBtn);

      fireEvent.keyDown(window, { key: 'Escape' });
      expect(handleClose).toHaveBeenCalledTimes(1);
    });

    it('does not render when isOpen is false', () => {
      render(
        <Modal isOpen={false} onClose={() => {}} title="Hidden">
          Content
        </Modal>
      );
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
  });

  describe('ConfirmDialog', () => {
    it('renders danger confirm button and handles actions', () => {
      const handleConfirm = vi.fn();
      const handleCancel = vi.fn();
      render(
        <ConfirmDialog
          isOpen={true}
          title="Delete Item"
          message="Are you sure you want to delete 1 item?"
          confirmLabel="Delete"
          cancelLabel="Cancel"
          onConfirm={handleConfirm}
          onCancel={handleCancel}
        />
      );

      expect(screen.getByText('Are you sure you want to delete 1 item?')).toBeInTheDocument();
      const confirmBtn = screen.getByRole('button', { name: 'Delete' });
      expect(confirmBtn).toHaveClass('btn-danger');

      fireEvent.click(confirmBtn);
      expect(handleConfirm).toHaveBeenCalledTimes(1);

      const cancelBtn = screen.getByRole('button', { name: 'Cancel' });
      fireEvent.click(cancelBtn);
      expect(handleCancel).toHaveBeenCalledTimes(1);
    });
  });

  describe('MarkerBadge', () => {
    it('renders clamped marker numbers', () => {
      const { rerender } = render(<MarkerBadge number={5} />);
      expect(screen.getByText('5')).toBeInTheDocument();

      rerender(<MarkerBadge number={150} />);
      expect(screen.getByText('99')).toBeInTheDocument();

      rerender(<MarkerBadge number={0} />);
      expect(screen.getByText('1')).toBeInTheDocument();
    });
  });
  describe('Navigation Rail and Interactive Focus States (LC-028, NFR-16)', () => {
    it('defines focus-visible styling for navigation rail items and buttons in components.css', () => {
      const cssPath = path.resolve(process.cwd(), 'src/styles/components.css');
      const css = fs.readFileSync(cssPath, 'utf-8');
      expect(css).toContain('.nav-rail-item:focus-visible');
      expect(css).toContain('.rail-capture-btn:focus-visible');
      expect(css).toContain('outline: 2px solid var(--color-accent)');
    });
  });
});
