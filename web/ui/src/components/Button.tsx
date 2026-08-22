import React, { ButtonHTMLAttributes } from 'react';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger';
  loading?: boolean;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  variant = 'secondary',
  loading = false,
  disabled,
  className = '',
  style,
  ...props
}) => {
  const variantClass = `btn-${variant}`;
  const loadingClass = loading ? 'loading' : '';
  const combinedClassName = `btn ${variantClass} ${loadingClass} ${className}`.trim();

  return (
    <button
      disabled={disabled || loading}
      className={combinedClassName}
      style={style}
      {...props}
    >
      {loading && <span>...</span>}
      {children}
    </button>
  );
};
