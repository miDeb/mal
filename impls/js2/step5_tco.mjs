import { createInterface } from "readline";
import { read_str } from "./reader.mjs";
import { pr_str } from "./printer.mjs";
import { compile } from "./compiler.mjs";
import { Vec } from "./types.mjs";
import { is_list } from "./types.mjs";
import { core } from "./core.mjs";

const rl = createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: true,
});

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

function log(value) {
  console.log(value);
  return value;
}

rl.setPrompt("user> ");
rl.prompt();
const env = core();
rep("(def! not (fn* (a) (if a false true)))", env);
rl.on("line", (line) => {
  console.log(rep(line, env));
  rl.prompt();
});
rl.on("close", () => {
  console.log();
});

function READ(input) {
  return read_str(input);
}
function PRINT(input) {
  return pr_str(input, true);
}

function rep(input, env) {
  let result;
  try {
    result = js_eval(compile(READ(input), env));
  } catch (e) {
    result = e.message;
  }
  return PRINT(result);
}
