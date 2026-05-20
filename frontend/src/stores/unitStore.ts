import { create } from 'zustand';

type UnitSystem = 'metric' | 'imperial';

interface UnitState {
  unitSystem: UnitSystem;
  setUnitSystem: (system: UnitSystem) => void;
  toggleUnitSystem: () => void;
}

export const useUnitStore = create<UnitState>((set) => ({
  unitSystem: 'metric',
  setUnitSystem: (system) => set({ unitSystem: system }),
  toggleUnitSystem: () =>
    set((s) => ({ unitSystem: s.unitSystem === 'metric' ? 'imperial' : 'metric' }),
  ),
}));
