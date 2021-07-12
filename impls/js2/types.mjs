export class Vec extends Array {}

export function is_list(value) {
  return Array.isArray(value) && !(value instanceof Vec);
}
export function is_vec(value) {
  return value instanceof Vec;
}
export function is_symbol(value) {
  return typeof value === "symbol";
}
export function is_map(value) {
  return value instanceof Map;
}
export function is_atom(value) {
  return value instanceof Atom;
}
export function is_macro(value) {
  return value?.is_macro ?? false;
}

export function eq(a, b) {
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) {
      return false;
    }
    for (let i = 0; i < a.length; i++) {
      if (!eq(a[i], b[i])) {
        return false;
      }
    }
    return true;
  } else if (a instanceof Map && b instanceof Map) {
    if (a.size !== b.size) {
      return false;
    }
    for (let k of a.keys()) {
      if (!eq(a.get(k), b.get(k))) {
        return false;
      }
    }
    return true;
  } else {
    return a === b;
  }
}

export class Atom {
  constructor(val) { this.val = val }
}
