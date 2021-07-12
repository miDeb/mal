import { read_str } from "./reader.mjs";
import { pr_str } from "./printer.mjs";
import { core } from "./core.mjs";
import { readline } from "./node_readline.mjs";
import { ret_val } from "./fn_calls.mjs";
import { compile_and_eval } from "./compiler.mjs";

function READ(input) {
  return read_str(input);
}
function PRINT(input) {
  return pr_str(input, true);
}

function rep(input, env) {
  let result;
  try {
    result = PRINT(compile_and_eval(READ(input), env));
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
    result = ret_val(compile_and_eval(prog, env));
  } catch (e) {
    result = e.message;
  }
  return result;
};
rep(
  `(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))`,
  env
);
rep("(def! not (fn* (a) (if a false true)))", env);
rep(
  "(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw \"odd number of forms to cond\")) (cons 'cond (rest (rest xs)))))))",
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
