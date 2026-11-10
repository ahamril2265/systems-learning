use std::env;
use std::process::{exit, Command};

fn main() {
    // Collect the words typed on the command line into a list.
    let args: Vec<String> = env::args().collect();

    // We need at least: minirun run <program>
    if args.len() < 3 || args[1] != "run" {
        eprintln!("usage: minirun run <command> [args...]");
        exit(2);
    }

    let program = &args[2];        // e.g. "ls"
    let program_args = &args[3..]; // e.g. ["-l"]

    println!("[minirun] starting: {} {:?}", program, program_args);

    // Start the program, wait for it to finish, and get its exit status.
    let result = Command::new(program).args(program_args).status();

    match result {
        Ok(status) => println!("[minirun] finished: {}", status),
        Err(error) => {
            eprintln!("[minirun] could not start '{}': {}", program, error);
            exit(1);
        }
    }
}
