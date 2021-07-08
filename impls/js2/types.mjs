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
