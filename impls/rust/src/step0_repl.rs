use rustyline::Editor;

fn main() {
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").ok();
    while let Ok(line) = rl.readline("user> ") {
        rl.add_history_entry(&line);

        let result = print(eval(read(line)));
        print!("{}", result);
    }
    rl.save_history("history.txt").unwrap();
}

fn read(input: String) -> String {
    input
}

fn eval(input: String) -> String {
    input
}

fn print(input: String) -> String {
    input
}
