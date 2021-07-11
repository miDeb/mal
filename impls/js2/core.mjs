import { pr_str } from "./printer.mjs";
import { eq } from "./types.mjs";
import { Atom } from "./types.mjs";
import { ret_val } from "./fn_calls.mjs";
import { readline } from "./node_readline.mjs";
import { read_str } from "./reader.mjs";
import { readFileSync } from "fs";
import { is_atom } from "./types.mjs";
import { call_fn } from "./fn_calls.mjs";
import { Vec, is_vec, is_list } from "./types.mjs";

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
  readline: () => ret_val(readline()),
  "read-string": (a) => ret_val(read_str(a)),
  slurp: (a) => ret_val(readFileSync(a, "utf-8")),
  atom: (a) => ret_val(new Atom(a)),
  "atom?": (a) => ret_val(is_atom(a)),
  deref: (a) => ret_val(a.val),
  "reset!": (atom, val) => ret_val((atom.val = val)),
  "swap!": (atom, fn, ...args) => {
    return ret_val((atom.val = call_fn(fn, [atom.val, ...args])));
  },
  cons: (element, list) => ret_val([element, ...list]),
  concat: (...lists) => {
    const result = [];
    for (const list of lists) {
      result.push(...list);
    }
    return ret_val(result);
  },
  vec: (arg) => {
    if (is_list(arg)) {
      return ret_val(Vec.from(arg));
    } else if (is_vec) {
      return ret_val(arg);
    } else {
      throw new Error(`expected list or vec, got ${arg}`);
    }
  },
});
