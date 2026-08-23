import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Global mock for Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === 'list_findings') return Promise.resolve([]);
    if (cmd === 'list_bundles') return Promise.resolve([]);
    if (cmd === 'get_access_key_status') return Promise.resolve({ exists: false, is_active: false, masked_prefix: null, issued_at: null });
    return Promise.resolve(null);
  }),
}));

