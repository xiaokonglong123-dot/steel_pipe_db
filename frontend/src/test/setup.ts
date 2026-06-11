/**
 * Test setup — runs before every test file.
 *
 * - Extends vitest assertions with @testing-library/jest-dom matchers
 *   (toBeInTheDocument, toHaveTextContent, etc.)
 * - Cleans up ReactDOM rendering between tests
 * - Configures mocks for browser APIs that jsdom doesn't implement
 */
import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

// Clean up ReactDOM tree after each test
afterEach(() => {
  cleanup();
});

// Mock matchMedia for Ant Design components that check viewport
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// localStorage may be unavailable in Node 26's jsdom — provide a mock instead.
const createLocalStorageMock = () => {
  const store = new Map<string, string>();
  return {
    getItem: (key: string): string | null => store.get(key) ?? null,
    setItem: (key: string, value: string): void => { store.set(key, value); },
    removeItem: (key: string): void => { store.delete(key); },
    clear: (): void => store.clear(),
    get length(): number { return store.size; },
    key: (index: number): string | null => [...store.keys()][index] ?? null,
  } as Storage;
};

Object.defineProperty(globalThis, 'localStorage', {
  configurable: true,
  value: createLocalStorageMock(),
});
