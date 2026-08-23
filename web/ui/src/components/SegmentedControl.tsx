import React, { useRef } from 'react';

export interface SegmentedControlOption<T extends string = string> {
  value: T;
  label: React.ReactNode;
  disabled?: boolean;
}

export interface SegmentedControlProps<T extends string = string> {
  id?: string;
  name?: string;
  options: SegmentedControlOption<T>[];
  value: T;
  onChange: (value: T) => void;
  disabled?: boolean;
  'aria-label'?: string;
  className?: string;
  style?: React.CSSProperties;
}

export function SegmentedControl<T extends string = string>({
  id,
  name,
  options,
  value,
  onChange,
  disabled = false,
  'aria-label': ariaLabel,
  className = '',
  style,
}: SegmentedControlProps<T>) {
  const containerRef = useRef<HTMLDivElement>(null);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>, currentIndex: number) => {
    if (disabled) return;

    const enabledOptions = options
      .map((opt, idx) => ({ opt, idx }))
      .filter(({ opt }) => !opt.disabled);

    if (enabledOptions.length <= 1) return;

    let targetIdx: number | null = null;

    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      const currentPos = enabledOptions.findIndex((item) => item.idx === currentIndex);
      const nextPos = (currentPos + 1) % enabledOptions.length;
      targetIdx = enabledOptions[nextPos].idx;
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      const currentPos = enabledOptions.findIndex((item) => item.idx === currentIndex);
      const prevPos = (currentPos - 1 + enabledOptions.length) % enabledOptions.length;
      targetIdx = enabledOptions[prevPos].idx;
    } else if (e.key === 'Home') {
      e.preventDefault();
      targetIdx = enabledOptions[0].idx;
    } else if (e.key === 'End') {
      e.preventDefault();
      targetIdx = enabledOptions[enabledOptions.length - 1].idx;
    }

    if (targetIdx !== null) {
      const targetOption = options[targetIdx];
      onChange(targetOption.value);
      const buttons = containerRef.current?.querySelectorAll<HTMLButtonElement>('button');
      if (buttons && buttons[targetIdx]) {
        buttons[targetIdx].focus();
      }
    }
  };

  return (
    <div
      ref={containerRef}
      id={id}
      role="radiogroup"
      aria-label={ariaLabel}
      className={`segmented-control ${className}`.trim()}
      style={style}
    >
      {options.map((option, index) => {
        const isSelected = option.value === value;
        const isDisabled = disabled || option.disabled;

        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            name={name}
            aria-checked={isSelected}
            aria-selected={isSelected}
            disabled={isDisabled}
            tabIndex={isSelected ? 0 : -1}
            onClick={() => {
              if (!isDisabled) onChange(option.value);
            }}
            onKeyDown={(e) => handleKeyDown(e, index)}
            className="segmented-control-option"
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
