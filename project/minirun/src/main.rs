use nix::mount::{mount, MsFlags};
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

    // In the PARENT: our next child will be born into a new PID namespace.
    if let Err(error) = unshare(CloneFlags::CLONE_NEWPID) {
        eprintln!("[minirun] cannot create PID namespace: {} (did you use sudo?)", error);
        exit(1);
    }

    println!("[minirun] parent pid on the host: {}", std::process::id());
    println!("[minirun] starting: {} {:?}", program, program_args);

    let mut cmd = Command::new(program);
    cmd.args(program_args);

    // In the CHILD: after fork, before exec.
    unsafe {
        cmd.pre_exec(|| {
            // Own hostname and own list of mounts.
            unshare(CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNS)?;
            sethostname("container")?;

            // Make all mounts private, so nothing we mount leaks to the host.
            mount(
                None::<&str>,
                "/",
                None::<&str>,
                MsFlags::MS_REC | MsFlags::MS_PRIVATE,
                None::<&str>,
            )?;

            // A fresh /proc that only shows this PID namespace.
            mount(
                Some("proc"),
                "/proc",
                Some("proc"),
                MsFlags::empty(),
                None::<&str>,
            )?;

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
