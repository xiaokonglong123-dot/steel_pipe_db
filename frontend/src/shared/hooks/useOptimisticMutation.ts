/**
 * Optimistic mutation hooks — React 19 useOptimistic + TanStack Query
 *
 * Provides instant UI feedback while mutations are in-flight.
 * If the server rejects the change, the UI rolls back automatically.
 */
import { useOptimistic, useCallback } from 'react';

/**
 * Hook for optimistic list mutations (create/update/delete).
 *
 * Usage:
 * ```tsx
 * const { optimisticItems, addOptimistic, removeOptimistic, updateOptimistic } =
 *   useOptimisticList({
 *     currentItems: items ?? [],
 *   });
 * ```
 */
export function useOptimisticList<T extends { id: number }>({
  currentItems,
}: {
  currentItems: T[];
}) {
  const [optimisticItems, setOptimisticItems] = useOptimistic(
    currentItems,
    (state, action: OptimisticAction<T>) => {
      switch (action.type) {
        case 'add':
          return [...state, action.item];
        case 'update':
          return state.map((item) =>
            item.id === action.item.id ? { ...item, ...action.item } : item,
          );
        case 'remove':
          return state.filter((item) => item.id !== action.id);
        case 'revert':
          return action.previous;
        default:
          return state;
      }
    },
  );

  const addOptimistic = useCallback(
    (item: T) => setOptimisticItems({ type: 'add', item }),
    [setOptimisticItems],
  );

  const updateOptimistic = useCallback(
    (item: Partial<T> & { id: number }) =>
      setOptimisticItems({ type: 'update', item }),
    [setOptimisticItems],
  );

  const removeOptimistic = useCallback(
    (id: number) => setOptimisticItems({ type: 'remove', id }),
    [setOptimisticItems],
  );

  return {
    optimisticItems,
    addOptimistic,
    updateOptimistic,
    removeOptimistic,
  };
}

type OptimisticAction<T> =
  | { type: 'add'; item: T }
  | { type: 'update'; item: Partial<T> & { id: number } }
  | { type: 'remove'; id: number }
  | { type: 'revert'; previous: T[] };

/**
 * Hook for optimistic detail mutations (update single item).
 *
 * Usage:
 * ```tsx
 * const { optimisticItem, updateOptimistic } = useOptimisticDetail({
 *   currentItem: item,
 * });
 * ```
 */
export function useOptimisticDetail<T extends { id: number }>({
  currentItem,
}: {
  currentItem: T | undefined;
}) {
  const [optimisticItem, setOptimisticItem] = useOptimistic(
    currentItem,
    (state, action: { type: 'update'; item: Partial<T> }) => {
      if (action.type === 'update' && state) {
        return { ...state, ...action.item } as T;
      }
      return state;
    },
  );

  const updateOptimistic = useCallback(
    (item: Partial<T>) => setOptimisticItem({ type: 'update', item }),
    [setOptimisticItem],
  );

  return {
    optimisticItem,
    updateOptimistic,
  };
}
