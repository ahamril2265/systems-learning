use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{chdir, chroot, sethostname};
use std::env;
use std::os::unix::process::CommandExt;
use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 || args[1] != "run" {
        eprintln!("usage: minirun run <image-folder> <command> [args...]");
        exit(2);
    }

    let rootfs = args[2].clone();   // our own copy, for the child to take
    let program = &args[3];
    let program_args = &args[4..];

    // In the PARENT: our next child will be born into a new PID namespace.
    if let Err(error) = unshare(CloneFlags::CLONE_NEWPID) {
        eprintln!("[minirun] cannot create PID namespace: {} (did you use sudo?)", error);
        exit(1);
    }

    println!("[minirun] parent pid on the host: {}", std::process::id());
    println!("[minirun] image: {}", rootfs);
    println!("[minirun] starting: {} {:?}", program, program_args);

    let mut cmd = Command::new(program);
    cmd.args(program_args);

    // In the CHILD: after fork, before exec.
    unsafe {
        cmd.pre_exec(move || {
            // 1. Own hostname and own list of mounts.
            unshare(CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNS)?;
            sethostname("container")?;

            // 2. Make all mounts private, so nothing leaks to the host.
            mount(
                None::<&str>,
                "/",
                None::<&str>,
                MsFlags::MS_REC | MsFlags::MS_PRIVATE,
                None::<&str>,
            )?;

            // 3. From now on, the image folder is "/".
            chroot(rootfs.as_str())?;

            // 4. Stand at the new root.
            chdir("/")?;

            // 5. A fresh /proc inside the image.
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
