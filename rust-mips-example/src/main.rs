use std::env::{args, var};

fn main() {
    let mut args = args();
    let binary_name = args.next().unwrap();
    println!("Run {}", binary_name);
    for arg in args {
        println!("{}={}", arg, var(arg.as_str()).unwrap_or_default());
    }
}
