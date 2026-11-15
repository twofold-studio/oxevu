const KEY = "oxevu.seek-seconds";

export function validSeekSeconds(value: number): boolean {
  return Number.isInteger(value) && value >= 1 && value <= 60;
}

export function readSeekSeconds(): number {
  try {
    const value = Number(localStorage.getItem(KEY));
    return validSeekSeconds(value) ? value : 5;
  } catch {
    return 5;
  }
}

export function saveSeekSeconds(value: number): boolean {
  if (!validSeekSeconds(value)) return false;
  try {
    localStorage.setItem(KEY, String(value));
    return true;
  } catch {
    return false;
  }
}
