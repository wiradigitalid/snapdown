import React from 'react';

export type BadgeVariant = 'success' | 'warning' | 'info' | 'neutral' | 'danger';

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
  children: React.ReactNode;
}

export const Badge: React.FC<BadgeProps> = ({
  variant = 'neutral',
  children,
  className = '',
  style,
  ...props
}) => {
  const badgeClass = `badge badge-${variant} ${className}`.trim();

  return (
    <span className={badgeClass} style={style} {...props}>
      {children}
    </span>
  );
};
