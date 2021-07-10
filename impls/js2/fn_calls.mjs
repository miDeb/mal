export function call_fn(fn, args) {
  let ret_val = fn.apply(fn, args);
  while (ret_val.tco) {
    ret_val = ret_val.fn.apply(ret_val.fn, ret_val.args);
  }
  return ret_val.value;
}

export function ret_val(val) {
  return {
    tco: false,
    value: val,
  };
}

export function ret_fn_call(fn, args) {
  return {
    tco: true,
    fn: fn,
    args: args,
  };
}
