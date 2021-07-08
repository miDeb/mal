import { createInterface } from "readline";
import { read_str } from "./reader.mjs";
import { pr_str } from "./printer.mjs";

const rl = createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: true,
});

const memoized_compilations = new Map();
function js_eval([fn_body, const_table]) {
  let fn = memoized_compilations.get(fn_body);
  if (!fn) {
    fn = Function(
      `"use strict";const const_table = arguments[0];${fn_body}`
    );
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
rl.on("line", (line) => {
  console.log(compiled_rep(line));
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

function compile(input) {
  return ["return const_table[0];", [input]];
}
function compiled_rep(input) {
  let result;
  try {
    result = js_eval(compile(READ(input)));
  } catch (e) {
    result = e.message;
  }
  return PRINT(result);
}
