export class Vec extends Array {}

class Reader {
  constructor(tokens) {
    this.tokens = tokens;
    this.position = 0;
  }
  next() {
    if (!this.hasTokens()) {
      throw new Error("unexpected EOF");
    }
    return this.tokens[this.position++];
  }
  peek() {
    if (!this.hasTokens()) {
      throw new Error("unexpected EOF");
    }
    return this.tokens[this.position];
  }

  hasTokens() {
    return this.position < this.tokens.length;
  }

  read_form() {
    switch (this.peek()) {
      case "(":
        return this.read_list(")");
      case "[":
        return Vec.from(this.read_list("]"));
      case "{":
        return this.read_map();
      case "'":
        this.next();
        return [Symbol.for("quote"), this.read_form()];
      case "`":
        this.next();
        return [Symbol.for("quasiquote"), this.read_form()];
      case "~":
        this.next();
        return [Symbol.for("unquote"), this.read_form()];
      case "~@":
        this.next();
        return [Symbol.for("splice-unquote"), this.read_form()];
      case "^":
        this.next();
        var meta = this.read_form();
        return [Symbol.for("with-meta"), this.read_form(), meta];
      case "@":
        this.next();
        return [Symbol.for("deref"), this.read_form()];
      default:
        return this.read_atom();
    }
  }
  read_map() {
    const map = new Map();
    const list = this.read_list("}");
    for (let index = 0; index < list.length; index += 2) {
      map.set(list[index], list[index + 1]);
    }
    return map;
  }
  read_list(end) {
    this.next();
    let list = [];
    while (this.peek()[0] != end) {
      list.push(this.read_form());
    }
    this.next();
    return list;
  }
  read_atom() {
    const token = this.next();
    if (token.match(/^-?[0-9]+$/)) {
      return parseInt(token, 10); // integer
    } else if (token.match(/^-?[0-9][0-9.]*$/)) {
      return parseFloat(token, 10); // float
    } else if (token.match(/^"(?:\\.|[^\\"])*"$/)) {
      return token
        .slice(1, token.length - 1)
        .replace(/\\(.)/g, (_, c) => (c === "n" ? "\n" : c));
    } else if (token[0] === '"') {
      throw new Error("unexpected EOF");
    } else if (token[0] === ":") {
      return "\u029e" + token.slice(1);
    } else if (token === "nil") {
      return null;
    } else if (token === "true") {
      return true;
    } else if (token === "false") {
      return false;
    } else {
      return Symbol.for(token); // symbol
    }
  }
}

function tokenize(input) {
  const token_regex =
    /[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)/g;
  let tokens = [];
  let match;
  while ((match = token_regex.exec(input)[1]) != "") {
    if (match[0] === ";") {
      continue;
    }
    tokens.push(match);
  }
  return tokens;
}

export function read_str(input) {
  const tokens = tokenize(input);
  const reader = new Reader(tokens);
  return reader.read_form();
}
