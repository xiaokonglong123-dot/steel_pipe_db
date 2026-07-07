/**
 * Shared components unit tests.
 *
 * Currently only ErrorBoundary exists in shared/components/.
 * Tests other components as they are added.
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';

// ─── ErrorBoundary ─────────────────────────────────────────────────

describe('ErrorBoundary', () => {
  it('renders children when no error', async () => {
    const { default: ErrorBoundary } = await import(
      '@/shared/components/ErrorBoundary'
    );
    render(
      <ErrorBoundary>
        <div>Safe Content</div>
      </ErrorBoundary>,
    );
    expect(screen.getByText('Safe Content')).toBeTruthy();
  });
});
