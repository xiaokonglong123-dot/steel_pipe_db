import { useUnitStore } from '../../stores/unitStore';

export function formatLength(mm: number, system: 'metric' | 'imperial'): string {
  if (system === 'imperial') {
    const inches = mm / 25.4;
    return `${inches.toFixed(2)} in`;
  }
  return `${mm.toFixed(1)} mm`;
}

export function formatWeight(kg: number, system: 'metric' | 'imperial'): string {
  if (system === 'imperial') {
    const lb = kg * 2.20462;
    return `${lb.toFixed(2)} lb`;
  }
  return `${kg.toFixed(2)} kg`;
}

export function formatDiameter(mm: number, system: 'metric' | 'imperial'): string {
  if (system === 'imperial') {
    const inches = mm / 25.4;
    return `${inches.toFixed(3)} in`;
  }
  return `${mm.toFixed(1)} mm`;
}

export function formatPressure(mpa: number, system: 'metric' | 'imperial'): string {
  if (system === 'imperial') {
    const psi = mpa * 145.038;
    return `${psi.toFixed(0)} psi`;
  }
  return `${mpa.toFixed(1)} MPa`;
}

export function useUnit() {
  const unitSystem = useUnitStore((s) => s.unitSystem);

  return {
    length: (mm: number) => formatLength(mm, unitSystem),
    weight: (kg: number) => formatWeight(kg, unitSystem),
    diameter: (mm: number) => formatDiameter(mm, unitSystem),
    pressure: (mpa: number) => formatPressure(mpa, unitSystem),
    unitSystem,
  };
}
