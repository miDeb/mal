import { Vec } from "./types.mjs";

export function pr_str(value, print_readably) {
  if (typeof value === "string") {
    if (value.startsWith("\u029e")) {
      return ":" + value.slice(1);
    } else if (print_readably) {
      return (
        '"' +
        value
          .replace(/\\/g, "\\\\")
          .replace(/"/g, '\\"')
          .replace(/\n/g, "\\n") +
        '"'
      );
    } else {
      return value;
    }
  } else if (typeof value === "number") {
    return value.toString();
  } else if (Array.isArray(value)) {
    const delimiters = value instanceof Vec ? "[]" : "()";
    return (
      delimiters[0] +
      value.map((e) => pr_str(e, print_readably)).join(" ") +
      delimiters[1]
    );
  } else if (value instanceof Map) {
    const components = [];
    for (let [k, v] of value) {
      components.push(pr_str(k, print_readably), pr_str(v, print_readably));
    }
    return "{" + components.join(" ") + "}";
  } else if (typeof value === "symbol") {
    return Symbol.keyFor(value);
  } else if (value === null) {
    return "nil";
  } else if (typeof value === "function") {
    return "#<function>";
  } else {
    return value.toString();
  }
}
