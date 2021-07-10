import { pr_str } from "./printer.mjs";
import { eq } from "./types.mjs";
import { is_list } from "./types.mjs";
import { ret_val } from "./fn_calls.mjs";

export const core = () => ({
  "<": (a, b) => ret_val(a < b),
  "<=": (a, b) => ret_val(a <= b),
  ">": (a, b) => ret_val(a > b),
  ">=": (a, b) => ret_val(a >= b),
  "+": (a, b) => ret_val(a + b),
  "-": (a, b) => ret_val(a - b),
  "*": (a, b) => ret_val(a * b),
  "/": (a, b) => ret_val(a / b),
  prn: (...args) => {
    console.log(args.map((a) => pr_str(a, true)).join(" "));
    return ret_val(null);
  },
  list: (...args) => ret_val(args),
  "list?": (arg) => ret_val(is_list(arg)),
  "empty?": (arg) => ret_val(arg.length === 0),
  count: (arg) => ret_val(arg?.length ?? 0),
  "=": (a, b) => ret_val(eq(a, b)),
  "pr-str": (...args) => ret_val(args.map((a) => pr_str(a, true)).join(" ")),
  str: (...args) => ret_val(args.map((a) => pr_str(a, false)).join("")),
  println: (...args) => {
    console.log(args.map((a) => pr_str(a, false)).join(" "));
    return ret_val(null);
  },
});
