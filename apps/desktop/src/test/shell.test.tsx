import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { App } from '../App';

describe('Desktop Shell', () => {
  it('app_renders_shell and displays settings heading', () => {
    render(<App />);
    const shell = screen.getByTestId('app-shell');
    expect(shell).toBeInTheDocument();
    expect(screen.getByText('Snapdown Settings')).toBeInTheDocument();
    expect(screen.getByText('Vault Path')).toBeInTheDocument();
  });
});
