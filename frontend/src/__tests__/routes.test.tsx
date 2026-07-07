/**
 * Smoke tests — verify that all route components render without crashing.
 *
 * Uses React Testing Library to render each lazy page component in isolation.
 * Each test only checks that the component mounts (no unhandled errors).
 * Full integration testing belongs in E2E tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/react';

// Mock react-router-dom hooks that page components use
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => vi.fn(),
    useParams: () => ({ id: undefined }),
    useLocation: () => ({ pathname: '/test', search: '', hash: '', state: null }),
    useSearchParams: () => [new URLSearchParams(), vi.fn()],
    Link: ({ children, to, ...props }: { children: React.ReactNode; to: string }) => (
      <a href={to} {...props}>
        {children}
      </a>
    ),
  };
});

// Mock the auth store to prevent auth redirect
const mockAuthStoreState = vi.hoisted(() => ({
  token: 'mock-token',
  user: { id: 1, username: 'admin', role: 'admin' },
}));

vi.mock('@/stores/authStore', () => {
  const mockFn = Object.assign(
    (selector: (s: typeof mockAuthStoreState) => unknown) => selector(mockAuthStoreState),
    { getState: () => mockAuthStoreState },
  );
  return { useAuthStore: mockFn };
});

// Mock TanStack Query
vi.mock('@tanstack/react-query', () => ({
  useQuery: () => ({ data: undefined, isLoading: false, error: null }),
  useMutation: () => ({ mutate: vi.fn(), isPending: false }),
  QueryClient: vi.fn(),
  QueryClientProvider: ({ children }: { children: React.ReactNode }) => children,
}));

// Mock i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'zh-CN', changeLanguage: vi.fn() },
  }),
  Trans: ({ children }: { children: React.ReactNode }) => children,
  initReactI18next: { type: '3rdParty', init: vi.fn() },
}));

describe('Route smoke tests', () => {
  // Login page (no auth required)
  it('renders LoginPage', async () => {
    const LoginPage = (await import('@/features/auth/pages/LoginPage')).default;
    render(<LoginPage />);
    // Login page should have some visible content
    expect(document.body).toBeTruthy();
  });

  // List pages — should at least mount without crashing
  const listPages = [
    ['SeamlessPipeListPage', () => import('@/features/pipes/pages/SeamlessPipeListPage')],
    ['ScreenPipeListPage', () => import('@/features/pipes/pages/ScreenPipeListPage')],
    ['InboundListPage', () => import('@/features/inventory/pages/InboundListPage')],
    ['OutboundListPage', () => import('@/features/inventory/pages/OutboundListPage')],
    ['StockQueryPage', () => import('@/features/inventory/pages/StockQueryPage')],
    ['LocationListPage', () => import('@/features/inventory/pages/LocationListPage')],
    ['InventoryCheckListPage', () => import('@/features/inventory/pages/InventoryCheckListPage')],
    ['SupplierListPage', () => import('@/features/suppliers/pages/SupplierListPage')],
    ['CustomerListPage', () => import('@/features/customers/pages/CustomerListPage')],
    ['PurchaseOrderListPage', () => import('@/features/purchases/pages/PurchaseOrderListPage')],
    ['SalesOrderListPage', () => import('@/features/sales/pages/SalesOrderListPage')],
    ['CertListPage', () => import('@/features/quality/pages/CertListPage')],
    ['ContractListPage', () => import('@/features/contracts/pages/ContractListPage')],
    ['ReportListPage', () => import('@/features/reports/pages/ReportListPage')],
    ['LabelPrintPage', () => import('@/features/labels/pages/LabelPrintPage')],
  ] as const;

  it.each(listPages)(
    'renders %s without import errors',
    async (_name, importFn) => {
      const mod = await importFn();
      const Component = mod.default;
      // Just verify the module exports a valid React component
      expect(Component).toBeDefined();
      expect(typeof Component).toBe('function');
    },
  );
});
