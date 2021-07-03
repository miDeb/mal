use crate::{env::Env, printer::pr_str, value::Value};

pub fn init_env(env: &mut Env) {
    env.set("+".to_string(), Value::Fn(|list| Ok(&list[0] + &list[1])));
    env.set("-".to_string(), Value::Fn(|list| Ok(&list[0] - &list[1])));
    env.set("*".to_string(), Value::Fn(|list| Ok(&list[0] * &list[1])));
    env.set("/".to_string(), Value::Fn(|list| &list[0] / &list[1]));
    env.set(
        "pr-str".to_string(),
        Value::Fn(|list| {
            let mut string = String::new();
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(item, &mut string, true).unwrap();
            }
            Ok(Value::String(string))
        }),
    );
    env.set(
        "str".to_string(),
        Value::Fn(|list| {
            let mut string = String::new();
            for item in list.iter() {
                pr_str(item, &mut string, false).unwrap();
            }
            Ok(Value::String(string))
        }),
    );
    env.set(
        "prn".to_string(),
        Value::Fn(|list| {
            let mut string = String::new();
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(item, &mut string, true).unwrap();
            }
            println!("{}", string);
            Ok(Value::Nil)
        }),
    );
    env.set(
        "println".to_string(),
        Value::Fn(|list| {
            let mut string = String::new();
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(item, &mut string, false).unwrap();
            }
            println!("{}", string);
            Ok(Value::Nil)
        }),
    );
    env.set(
        "list".to_string(),
        Value::Fn(|list| Ok(Value::List(list.to_vec()))),
    );
    env.set(
        "list?".to_string(),
        Value::Fn(|list| Ok(Value::Bool(matches!(list[0], Value::List(_))))),
    );
    env.set(
        "empty?".to_string(),
        Value::Fn(|list| {
            Ok(Value::Bool(
                list[0]
                    .try_as_list_or_vec()
                    .map(|l| l.is_empty())
                    .unwrap_or(true),
            ))
        }),
    );
    env.set(
        "count".to_string(),
        Value::Fn(|list| {
            Ok(Value::Number(
                list[0].try_as_list_or_vec().map(|l| l.len()).unwrap_or(0) as i32,
            ))
        }),
    );
    env.set(
        "=".to_string(),
        Value::Fn(|list| Ok(Value::Bool(list[0] == list[1]))),
    );
    env.set(
        "<".to_string(),
        Value::Fn(|list| Ok(Value::Bool(list[0] < list[1]))),
    );
    env.set(
        ">".to_string(),
        Value::Fn(|list| Ok(Value::Bool(list[0] > list[1]))),
    );
    env.set(
        "<=".to_string(),
        Value::Fn(|list| Ok(Value::Bool(list[0] <= list[1]))),
    );
    env.set(
        ">=".to_string(),
        Value::Fn(|list| Ok(Value::Bool(list[0] >= list[1]))),
    );
}
