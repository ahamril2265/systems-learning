use nix::sched::{unshare, CloneFlags};
use nix::unistd::sethostname;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args[1] != "run" {
        eprintln!("usage: minirun run <command> [args...]");
        exit(2);
    }

    let program = &args[2];
    let program_args = &args[3..];

    println!("[minirun] starting: {} {:?}", program, program_args);

    let mut cmd = Command::new(program);
    cmd.args(program_args);

    // This code runs in the child: after fork, before exec.
    unsafe {
        cmd.pre_exec(|| {
            unshare(CloneFlags::CLONE_NEWUTS)?; // get our own UTS namespace
            sethostname("container")?;          // rename only our copy
            Ok(())
        });
    }

    match cmd.status() {
        Ok(status) => println!("[minirun] finished: {}", status),
        Err(error) => {
            eprintln!("[minirun] could not start '{}': {}", program, error);
            exit(1);
        }
    }
}
