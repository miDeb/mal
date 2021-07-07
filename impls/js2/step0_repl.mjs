import { createInterface } from "readline";

const rl = createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: true,
});

function js_eval(fn_body) {
  return Function(`"use strict";${fn_body}`)();
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
  return input;
}
function EVAL(input) {
  return input;
}
function PRINT(input) {
  return input;
}
function rep(input) {
  return PRINT(EVAL(READ(input)));
}

function compile(input) {
  return (
    'return "' + input.replaceAll("\\", "\\\\").replaceAll('"', '\\"') + '";'
  );
}
function compiled_rep(input) {
  return PRINT(js_eval(compile(READ(input))));
}
