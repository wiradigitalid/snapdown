import React from 'react';

export interface MarkdownProps {
  content: string;
  className?: string;
}

export const Markdown: React.FC<MarkdownProps> = ({ content, className }) => {
  const lines = content.split('\n');

  return (
    <div
      className={className}
      style={{
        fontFamily: 'var(--font-ui)',
        fontSize: 'var(--text-sm)',
        color: 'var(--color-text)',
        lineHeight: 1.6,
      }}
    >
      {lines.map((line, index) => {
        if (line.startsWith('# ')) {
          return (
            <h1 key={index} style={{ fontSize: 'var(--text-xl)', margin: 'var(--space-4) 0 var(--space-2)' }}>
              {line.replace('# ', '')}
            </h1>
          );
        }
        if (line.startsWith('## ')) {
          return (
            <h2 key={index} style={{ fontSize: 'var(--text-lg)', margin: 'var(--space-3) 0 var(--space-2)' }}>
              {line.replace('## ', '')}
            </h2>
          );
        }
        if (line.startsWith('### ')) {
          return (
            <h3 key={index} style={{ fontSize: 'var(--text-base)', margin: 'var(--space-2) 0 var(--space-1)' }}>
              {line.replace('### ', '')}
            </h3>
          );
        }
        if (line.startsWith('![')) {
          const match = line.match(/!\[(.*?)\]\((.*?)\)/);
          if (match) {
            const [, alt, src] = match;
            return (
              <div key={index} style={{ margin: 'var(--space-3) 0' }}>
                <img
                  src={src}
                  alt={alt}
                  style={{ maxWidth: '100%', height: 'auto', borderRadius: 'var(--radius-md)' }}
                />
              </div>
            );
          }
        }
        if (line.trim() === '') {
          return <div key={index} style={{ height: 'var(--space-2)' }} />;
        }
        return (
          <p key={index} style={{ margin: 'var(--space-1) 0' }}>
            {line}
          </p>
        );
      })}
    </div>
  );
};
