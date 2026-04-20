use tenet::run;

fn main() {
    let code = run(std::env::args_os());
    std::process::exit(code);
}
