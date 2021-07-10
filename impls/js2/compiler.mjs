import { ret_val, ret_fn_call, call_fn } from "./fn_calls.mjs";
import { Vec, is_vec } from "./types.mjs";
import { is_map } from "./types.mjs";
import { is_symbol } from "./types.mjs";
import { is_list } from "./types.mjs";

export function compile(input, global_env) {
  const compiler = new Compiler(global_env);
  // Maybe TCO here too?
  compiler.compile(input, (result) => `return ${result};`);
  return compiler.finish();
}

const ignore_value = (v) => `${v};\n`;
const return_result = (v) => `return ret_val(${v});\n`;

class Compiler {
  compiled = "";
  constants = [];
  tmp_count = 0;

  constructor(global_env) {
    this.constants.push(global_env);
    this.constants.push(Vec);
    this.emit(
      "{\nconst env = constants[0];\nconst Vec = constants[1];\nconst is_truthy = (v) => (v !== null && v !== false);\n"
    );
    this.constants.push(ret_val);
    this.emit("const ret_val = constants[2];\n");
    this.constants.push(ret_fn_call);
    this.emit("const ret_fn_call = constants[3];\n");
    this.constants.push(call_fn);
    this.emit("const call_fn = constants[4];\n");
  }

  create_scope() {
    // we can probably omit the creation of new scopes in some cases. Would that be useful?
    let tmp = this.emit_tmp();
    this.emit(tmp + " = env;\n");
    this.emit(`{\nlet env = Object.create(${tmp});\n`);
  }

  end_scope() {
    this.emit("\n}\n");
  }

  emit(str) {
    this.compiled += str;
  }

  emit_constant(value) {
    // TODO: how much sense does this handling of constants make?
    // We surely could inline numbers into the compiled code.
    // Are there constants where the current approach is useful? Strings probably?
    // Not inlining constants means that the compiled code is more reusable, but I don't know if
    // that will have any effect at some point.
    this.constants.push(value);
    return this.constants.length - 1;
  }

  emit_tmp() {
    let tmp = "v" + this.tmp_count++;
    this.emit(`let ${tmp};\n`);
    tmp = new String(tmp);
    Object.defineProperty(tmp, "assign", {
      value: (v) => `${tmp} = ${v};\n`,
      writable: false,
    });
    return tmp;
  }

  emit_tmp_assigned(value, use_let = false) {
    let tmp = "v" + this.tmp_count++;
    this.emit(`${use_let ? "let" : "const"} ${tmp} = ${value};\n`);
    tmp = new String(tmp);
    Object.defineProperty(tmp, "assign", {
      value: (v) => `${tmp} = ${v};\n`,
      writable: false,
    });
    return tmp;
  }

  emit_block(in_block) {
    this.emit("{\n");
    in_block();
    this.emit("}\n");
  }

  finish() {
    this.emit("\n}");
    return [this.compiled, this.constants];
  }

  compile(node, then) {
    if (is_list(node)) {
      this.compile_list(node, then);
    } else if (is_symbol(node)) {
      this.compile_symbol(node, then);
    } else if (is_vec(node)) {
      this.compile_vec(node, then);
    } else if (is_map(node)) {
      this.compile_map(node, then);
    } else {
      this.compile_constant(node, then);
    }
  }

  compile_constant(value, then) {
    let index = this.emit_constant(value);
    this.emit(then("constants[" + index + "]"));
  }

  compile_symbol(sym, then) {
    const symbol = Symbol.keyFor(sym);
    const tmp = this.emit_tmp_assigned(`env["${symbol}"]`);
    this.emit(
      `if (typeof ${tmp} === "undefined") throw new Error("${symbol} not found");\n`
    );
    this.emit(then(tmp));
  }

  compile_list(list, then) {
    if (list.length == 0) {
      this.emit(then("[]"));
    }
    // Special forms
    else if (is_symbol(list[0]) && Symbol.keyFor(list[0]) == "def!") {
      const key = Symbol.keyFor(list[1]);
      const tmp = this.emit_tmp();
      this.compile(list[2], tmp.assign);
      this.emit(then(`env["${key}"] = ${tmp}`));
    } else if (is_symbol(list[0]) && Symbol.keyFor(list[0]) == "let*") {
      this.create_scope();
      let bindings = list[1];
      const tmp = this.emit_tmp();
      for (let index = 0; index < bindings.length; index += 2) {
        const key = Symbol.keyFor(bindings[index]);
        this.compile(bindings[index + 1], tmp.assign);
        this.emit(`env["${key}"] = ${tmp};\n`);
      }
      this.compile(list[2], then);
      this.end_scope();
    } else if (is_symbol(list[0]) && Symbol.keyFor(list[0]) == "do") {
      for (let index = 1; index < list.length - 1; index++) {
        this.compile(list[index], ignore_value);
      }
      this.compile(list[list.length - 1], then);
    } else if (is_symbol(list[0]) && Symbol.keyFor(list[0]) == "if") {
      let condition = this.emit_tmp();
      this.compile(list[1], condition.assign);
      this.emit(`if (is_truthy(${condition})){\n`);
      this.compile(list[2], then);
      this.emit("} else {\n");
      this.compile(list[3] ?? null, then);
      this.emit("}\n");
    } else if (is_symbol(list[0]) && Symbol.keyFor(list[0]) == "fn*") {
      let args = list[1];
      let body = list[2];
      // TODO: the current mechanism requires us to create an unnecessary temporary variable :(
      let tmp = this.emit_tmp();
      const env_tmp = this.emit_tmp_assigned("env");
      this.emit(
        `${tmp} = function() {\nconst env = Object.create(${env_tmp});\n`
      );
      for (let index = 0; index < args.length; index++) {
        let symbol = Symbol.keyFor(args[index]);
        if (symbol === "&") {
          symbol = Symbol.keyFor(args[index + 1]);
          this.emit(`env["${symbol}"] = [...arguments].slice(${index});\n`);
          break;
        } else {
          this.emit(`env["${symbol}"] = arguments[${index}];\n`);
        }
      }
      this.compile(body, return_result);
      this.emit("};\n");
      this.emit(then(tmp));
    }
    // Function call
    else {
      const fn = this.emit_tmp();
      this.compile(list[0], fn.assign);
      const args = this.emit_tmp_assigned("[]");
      for (const arg of list.slice(1)) {
        this.compile(arg, (compiled) => `${args}.push(${compiled});\n`);
      }
      // Check if we can do tco
      if (then === return_result) {
        this.emit(`return ret_fn_call(${fn}, ${args})`);
      } else {
        this.emit(then(`call_fn(${fn}, ${args})`));
      }
    }
  }

  compile_vec(list, then) {
    const temporaries = [];
    for (const elem of list) {
      const tmp = this.emit_tmp();
      this.compile(elem, tmp.assign);
      temporaries.push(tmp);
    }
    this.emit(then(`Vec.from([${temporaries.join(", ")}])`));
  }

  compile_map(map, then) {
    const tmp = this.emit_tmp_assigned("new Map()");
    for (const [key, value] of map) {
      this.compile(value, (compiled) => {
        return `${tmp}.set("${key}", ${compiled});`;
      });
    }
    this.emit(then(tmp));
  }
}
