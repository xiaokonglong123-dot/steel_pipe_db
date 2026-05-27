/**
 * Unit system preference management — metric / imperial
 *
 * The user's unit system preference is persisted to localStorage,
 * feature modules read it from useUnitStore for display conversion.
 */
import { create } from 'zustand';

export type UnitSystem = 'metric' | 'imperial';

interface UnitState {
  unitSystem: UnitSystem;
  setUnitSystem: (system: UnitSystem) => void;
}

const LS_KEY = 'unit_system';

export const useUnitStore = create<UnitState>((set) => ({
  unitSystem: (localStorage.getItem(LS_KEY) as UnitSystem) || 'metric',

  setUnitSystem: (unitSystem) => {
    localStorage.setItem(LS_KEY, unitSystem);
    set({ unitSystem });
  },
}));
