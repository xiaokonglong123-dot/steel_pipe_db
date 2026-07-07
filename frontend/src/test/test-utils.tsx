/**
 * Shared test utilities for frontend unit tests.
 *
 * Provides:
 * - renderWithProviders — wraps components with necessary providers (Router, QueryClient, ConfigProvider)
 * - createMockQueryClient — creates a fresh QueryClient for each test
 * - mockAuthStore — helper to pre-populate auth state in tests
 */
import { type ReactNode } from 'react';
import { render, type RenderOptions, type RenderResult } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ConfigProvider } from 'antd';
import { useAuthStore } from '@/stores/authStore';
import type { UserInfo } from '@/types';

/** Create a fresh QueryClient for testing with safe defaults */
export function createMockQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
        staleTime: 0,
      },
    },
  });
}

/** Default test user */
export const testUser: UserInfo = {
  id: 1,
  username: 'admin',
  display_name: 'Admin',
  role: 'admin',
  email: 'admin@test.local',
};

interface RenderWithProvidersOptions extends Omit<RenderOptions, 'wrapper'> {
  initialEntries?: string[];
  queryClient?: QueryClient;
}

/**
 * Render a component wrapped with all necessary providers.
 *
 * Usage:
 * ```tsx
 * renderWithProviders(<MyComponent />, { initialEntries: ['/pipes'] })
 * ```
 */
export function renderWithProviders(
  ui: ReactNode,
  options: RenderWithProvidersOptions = {},
): RenderResult {
  const {
    initialEntries = ['/'],
    queryClient = createMockQueryClient(),
    ...renderOptions
  } = options;

  function Wrapper({ children }: { children: ReactNode }) {
    return (
      <ConfigProvider>
        <QueryClientProvider client={queryClient}>
          <MemoryRouter initialEntries={initialEntries}>
            {children}
          </MemoryRouter>
        </QueryClientProvider>
      </ConfigProvider>
    );
  }

  return render(ui, { wrapper: Wrapper, ...renderOptions });
}

/** Set the auth store to a logged-in state for tests that require authentication */
export function setAuthInStore(user: UserInfo = testUser, token = 'test-token'): void {
  useAuthStore.setState({ user, token });
}

/** Clear auth store state */
export function clearAuthStore(): void {
  useAuthStore.setState({ user: null, token: null });
}
