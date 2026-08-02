import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock the auth store before importing Can
vi.mock('@/stores/authStore', () => ({
  useAuthStore: (selector: (s: unknown) => unknown) =>
    selector({
      user: {
        id: 1,
        username: 'admin',
        display_name: 'Admin',
        role: 'admin',
        permissions: ['pipe.read', 'pipe.write', 'inventory.inbound'],
      },
    }),
}));

describe('Can', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it('renders children when permission is satisfied', async () => {
    const { Can } = await import('@/shared/components/Can');
    render(
      <Can permission="pipe.write">
        <button>Secret Button</button>
      </Can>,
    );
    expect(screen.getByText('Secret Button')).toBeTruthy();
  });

  it('hides children when permission is missing', async () => {
    const { Can } = await import('@/shared/components/Can');
    render(
      <Can permission="finance.pay">
        <button>Secret Button</button>
      </Can>,
    );
    expect(screen.queryByText('Secret Button')).toBeNull();
  });

  it('requires ALL permissions when given an array', async () => {
    const { Can } = await import('@/shared/components/Can');
    render(
      <Can permission={['pipe.read', 'inventory.inbound']}>
        <button>Both Held</button>
      </Can>,
    );
    expect(screen.getByText('Both Held')).toBeTruthy();
  });

  it('hides when only SOME of the array permissions are held', async () => {
    const { Can } = await import('@/shared/components/Can');
    render(
      <Can permission={['pipe.read', 'finance.pay']}>
        <button>Partial Held</button>
      </Can>,
    );
    expect(screen.queryByText('Partial Held')).toBeNull();
  });

  it('renders children when no permission prop given', async () => {
    const { Can } = await import('@/shared/components/Can');
    render(
      <Can>
        <span>Always Visible</span>
      </Can>,
    );
    expect(screen.getByText('Always Visible')).toBeTruthy();
  });

  it('renders fallback instead of children when denied', async () => {
    const { Can } = await import('@/shared/components/Can');
    render(
      <Can permission="finance.pay" fallback={<span>No Access</span>}>
        <button>Hidden</button>
      </Can>,
    );
    expect(screen.getByText('No Access')).toBeTruthy();
    expect(screen.queryByText('Hidden')).toBeNull();
  });
});
