import React, { useEffect, useRef } from 'react';

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  onClick: () => void;
  separator?: boolean;
}

export interface ContextMenuProps {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
  className?: string;
  style?: React.CSSProperties;
}

export const ContextMenu: React.FC<ContextMenuProps> = ({
  x,
  y,
  items,
  onClose,
  className = '',
  style,
}) => {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handlePointerDownOutside = (e: MouseEvent | TouchEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    const handleScrollOrResize = () => {
      onClose();
    };

    document.addEventListener('pointerdown', handlePointerDownOutside, true);
    document.addEventListener('keydown', handleKeyDown, true);
    window.addEventListener('resize', handleScrollOrResize);
    window.addEventListener('scroll', handleScrollOrResize, true);

    return () => {
      document.removeEventListener('pointerdown', handlePointerDownOutside, true);
      document.removeEventListener('keydown', handleKeyDown, true);
      window.removeEventListener('resize', handleScrollOrResize);
      window.removeEventListener('scroll', handleScrollOrResize, true);
    };
  }, [onClose]);

  // Adjust positioning so menu does not overflow viewport boundaries
  const adjustedPosition = (() => {
    if (typeof window === 'undefined') return { left: x, top: y };
    const menuWidth = 230;
    const menuHeight = items.length * 36 + 16;
    const padding = 8;

    let left = x;
    let top = y;

    if (left + menuWidth > window.innerWidth - padding) {
      left = Math.max(padding, window.innerWidth - menuWidth - padding);
    }
    if (top + menuHeight > window.innerHeight - padding) {
      top = Math.max(padding, window.innerHeight - menuHeight - padding);
    }

    return { left, top };
  })();

  return (
    <div
      ref={menuRef}
      data-testid="context-menu"
      className={`context-menu context-menu-container ${className}`.trim()}
      style={{
        position: 'fixed',
        left: `${adjustedPosition.left}px`,
        top: `${adjustedPosition.top}px`,
        ...style,
      }}
      onClick={(e) => e.stopPropagation()}
      onContextMenu={(e) => {
        e.preventDefault();
        e.stopPropagation();
      }}
    >
      {items.map((item, index) => {
        if (item.separator) {
          return (
            <div
              key={`sep-${index}`}
              data-testid="context-menu-separator"
              className="context-menu-divider"
            />
          );
        }

        return (
          <button
            key={item.id}
            type="button"
            data-testid={`context-menu-item-${item.id}`}
            disabled={item.disabled}
            className={`context-menu-btn ${item.danger ? 'danger' : ''}`.trim()}
            onClick={() => {
              if (!item.disabled) {
                item.onClick();
                onClose();
              }
            }}
          >
            <span
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 'var(--space-2)',
                flex: 1,
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}
            >
              {item.icon && (
                <span style={{ display: 'inline-flex', flexShrink: 0, fontSize: '1rem', lineHeight: 1 }}>
                  {item.icon}
                </span>
              )}
              <span style={{ fontWeight: 500 }}>{item.label}</span>
            </span>
            {item.shortcut && (
              <span className="context-menu-shortcut">
                {item.shortcut}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
};
