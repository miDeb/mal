import { pr_str } from "./printer.mjs";
import { eq } from "./types.mjs";
import { is_list } from "./types.mjs";

export const core = () => ({
  "<": (a, b) => a < b,
  "<=": (a, b) => a <= b,
  ">": (a, b) => a > b,
  ">=": (a, b) => a >= b,
  "+": (a, b) => a + b,
  "-": (a, b) => a - b,
  "*": (a, b) => a * b,
  "/": (a, b) => a / b,
  prn: (...args) => {
    console.log(args.map((a) => pr_str(a, true)).join(" "));
    return null;
  },
  list: (...args) => args,
  "list?": is_list,
  "empty?": (arg) => arg.length === 0,
  count: (arg) => arg?.length ?? 0,
  "=": eq,
  "pr-str": (...args) => args.map((a) => pr_str(a, true)).join(" "),
  str: (...args) => args.map((a) => pr_str(a, false)).join(""),
  println: (...args) => {
    console.log(args.map((a) => pr_str(a, false)).join(" "));
    return null;
  },
});
