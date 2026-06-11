/**
 * Non-blocking mutation hooks — React 19 useTransition
 *
 * Wraps mutations with useTransition to keep the UI responsive during
 * server state changes. Shows loading states without blocking interactions.
 */
import { useTransition, useCallback } from 'react';
import { useQueryClient, type QueryKey } from '@tanstack/react-query';

interface UseTransitionMutationOptions<TData, TVariables> {
  mutationFn: (variables: TVariables) => Promise<TData>;
  queryKey?: QueryKey;
  onSuccess?: (data: TData, variables: TVariables) => void;
  onError?: (error: Error, variables: TVariables) => void;
}

/**
 * Hook for non-blocking mutations with automatic query invalidation.
 *
 * Usage:
 * ```tsx
 * const { execute, isPending } = useTransitionMutation({
 *   mutationFn: (data) => api.create(data),
 *   queryKey: ['items'],
 *   onSuccess: () => message.success('Created'),
 * });
 *
 * await execute({ name: 'new item' });
 * ```
 */
export function useTransitionMutation<TData, TVariables>({
  mutationFn,
  queryKey,
  onSuccess,
  onError,
}: UseTransitionMutationOptions<TData, TVariables>) {
  const [isPending, startTransition] = useTransition();
  const queryClient = useQueryClient();

  const execute = useCallback(
    async (variables: TVariables) => {
      startTransition(async () => {
        try {
          const data = await mutationFn(variables);

          if (queryKey) {
            await queryClient.invalidateQueries({ queryKey });
          }

          onSuccess?.(data, variables);
        } catch (error) {
          onError?.(error as Error, variables);
        }
      });
    },
    [mutationFn, queryKey, queryClient, onSuccess, onError],
  );

  return {
    execute,
    isPending,
  };
}

/**
 * Hook for non-blocking optimistic mutations.
 * Combines useTransition with useOptimistic for instant UI feedback.
 */
export function useTransitionOptimistic<TData, TVariables>({
  mutationFn,
  queryKey,
  optimisticUpdate,
  onMutate,
  onSuccess,
  onError,
}: UseTransitionMutationOptions<TData, TVariables> & {
  optimisticUpdate: (variables: TVariables) => void;
  onMutate?: (variables: TVariables) => Promise<unknown> | unknown;
  onSettled?: () => void;
}) {
  const [isPending, startTransition] = useTransition();
  const queryClient = useQueryClient();

  const execute = useCallback(
    async (variables: TVariables) => {
      // Apply optimistic update immediately
      optimisticUpdate(variables);

      startTransition(async () => {
        let context: unknown;
        try {
          // Cancel outgoing queries for fresh data
          if (queryKey) {
            await queryClient.cancelQueries({ queryKey });
            context = await onMutate?.(variables);
          }

          const data = await mutationFn(variables);

          if (queryKey) {
            await queryClient.invalidateQueries({ queryKey });
          }

          onSuccess?.(data, variables);
        } catch (error) {
          // Revert optimistic update on error
          if (context && queryKey) {
            queryClient.setQueryData(queryKey, (context as { previous: unknown }).previous);
          }
          onError?.(error as Error, variables);
        }
      });
    },
    [
      mutationFn,
      queryKey,
      queryClient,
      optimisticUpdate,
      onMutate,
      onSuccess,
      onError,
    ],
  );

  return {
    execute,
    isPending,
  };
}
