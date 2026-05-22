// 单位制偏好 — 公制 / 英制，与 localStorage 双向同步
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
