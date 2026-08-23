import React, { useEffect, useRef, useState } from 'react';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
}

export const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
}) => {
  const modalRef = useRef<HTMLDivElement | null>(null);
  const previouslyFocusedElementRef = useRef<HTMLElement | null>(null);
  const [isClosing, setIsClosing] = useState<boolean>(false);

  // Store previously focused element and restore on close
  useEffect(() => {
    if (isOpen) {
      previouslyFocusedElementRef.current = document.activeElement as HTMLElement | null;
      setIsClosing(false);

      // Focus first focusable element inside modal or modal container itself
      const timer = setTimeout(() => {
        if (modalRef.current) {
          const focusable = modalRef.current.querySelectorAll<HTMLElement>(
            'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
          );
          if (focusable.length > 0) {
            focusable[0].focus();
          } else {
            modalRef.current.focus();
          }
        }
      }, 0);

      return () => clearTimeout(timer);
    } else {
      setIsClosing(false);
      if (previouslyFocusedElementRef.current) {
        previouslyFocusedElementRef.current.focus();
        previouslyFocusedElementRef.current = null;
      }
    }
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        setIsClosing(true);
        onClose();
        return;
      }

      if (e.key === 'Tab' && modalRef.current) {
        const focusable = Array.from(
          modalRef.current.querySelectorAll<HTMLElement>(
            'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"]):not([disabled])'
          )
        );

        if (focusable.length === 0) {
          e.preventDefault();
          modalRef.current.focus();
          return;
        }

        const first = focusable[0];
        const last = focusable[focusable.length - 1];

        if (e.shiftKey) {
          if (document.activeElement === first || document.activeElement === modalRef.current) {
            e.preventDefault();
            last.focus();
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault();
            first.focus();
          }
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen && !isClosing) return null;

  return (
    <div
      className="modal-overlay"
      data-state={isClosing ? 'closing' : 'open'}
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 'var(--z-modal)',
        backgroundColor: 'var(--color-overlay-scrim)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 'var(--space-4)',
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          setIsClosing(true);
          onClose();
        }
      }}
    >
      <div
        ref={modalRef}
        className="modal-content"
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
        data-state={isClosing ? 'closing' : 'open'}
        style={{
          backgroundColor: 'var(--color-surface)',
          color: 'var(--color-text)',
          borderRadius: 'var(--radius-md)',
          boxShadow: 'var(--shadow-raised)',
          border: '1px solid var(--color-border)',
          width: '100%',
          maxWidth: '32rem',
          padding: 'var(--space-5)',
          display: 'flex',
          flexDirection: 'column',
          gap: 'var(--space-4)',
          outline: 'none',
        }}
      >
        {title && (
          <h2
            style={{
              margin: 0,
              fontFamily: 'var(--font-ui)',
              fontSize: 'var(--text-lg)',
              fontWeight: 600,
            }}
          >
            {title}
          </h2>
        )}
        <div>{children}</div>
      </div>
    </div>
  );
};
