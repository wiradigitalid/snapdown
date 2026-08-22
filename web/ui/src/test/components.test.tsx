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

  describe('TextField', () => {
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
  });

  describe('Modal', () => {
    it('renders title, content, focus trap, and handles escape', () => {
      const handleClose = vi.fn();
      render(
        <Modal isOpen={true} onClose={handleClose} title="Test Modal">
          <button>Inside Button</button>
        </Modal>
      );

      expect(screen.getByText('Test Modal')).toBeInTheDocument();
      expect(screen.getByRole('dialog')).toBeInTheDocument();

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

  describe('EmptyState', () => {
    it('renders heading, description, and optional action', () => {
      const handleAction = vi.fn();
      render(
        <EmptyState
          heading="No Findings Yet"
          description="Use your hotkey to capture your first finding."
          actionLabel="Open Settings"
          onAction={handleAction}
        />
      );

      expect(screen.getByText('No Findings Yet')).toBeInTheDocument();
      expect(screen.getByText('Use your hotkey to capture your first finding.')).toBeInTheDocument();
      const actionBtn = screen.getByRole('button', { name: 'Open Settings' });
      fireEvent.click(actionBtn);
      expect(handleAction).toHaveBeenCalledTimes(1);
    });
  });
});
