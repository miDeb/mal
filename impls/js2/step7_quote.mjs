import { read_str } from "./reader.mjs";
import { pr_str } from "./printer.mjs";
import { compile } from "./compiler.mjs";
import { core } from "./core.mjs";
import { readline } from "./node_readline.mjs";
import { ret_val } from "./fn_calls.mjs";

const memoized_compilations = new Map();
function js_eval([fn_body, const_table]) {
  let fn = memoized_compilations.get(fn_body);
  if (!fn) {
    //console.log(fn_body);
    fn = Function(`"use strict";const constants = arguments[0];${fn_body}`);
    memoized_compilations.set(fn_body, fn);
  }
  return fn(const_table);
}

function READ(input) {
  return read_str(input);
}
function PRINT(input) {
  return pr_str(input, true);
}

function rep(input, env) {
  let result;
  try {
    result = PRINT(js_eval(compile(READ(input), env)));
  } catch (e) {
    result = e.message;
  }
  return result;
}

let input;
const env = core();
env["eval"] = (prog) => {
  let result;
  try {
    result = ret_val(js_eval(compile(prog, env)));
  } catch (e) {
    result = e.message;
  }
  return result;
};
rep(
  `(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))`,
  env
);
env["*ARGV*"] = process.argv.slice(3);
if (process.argv.length > 2) {
  rep(`(load-file "${process.argv[2]}")`, env);
} else {
  while ((input = readline()) !== null) {
    console.log(rep(input, env));
  }
}
