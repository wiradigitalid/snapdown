import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Global mock for Tauri invoke and convertFileSrc
vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: vi.fn().mockImplementation((filePath: string) => `asset://localhost/${encodeURIComponent(filePath)}`),
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === 'list_findings') return Promise.resolve([]);
    if (cmd === 'list_bundles') return Promise.resolve([]);
    if (cmd === 'get_access_key_status') return Promise.resolve({ exists: false, is_active: false, masked_prefix: null, issued_at: null });
    return Promise.resolve(null);
  }),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}));
