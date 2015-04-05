use std::env;
use std::io::Write;
use std::process;

fn read_print_file(path: &str) {
    ;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut stderr = std::io::stderr();
    if args.len() != 2 {
        writeln!(&mut stderr, "Usage: rexdump <file>").unwrap();
        process::exit(1);
    }
    read_print_file(&args[0]);
}
