use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{chdir, chroot, sethostname};
use std::env;
use std::fs;
use std::os::unix::process::CommandExt;
use std::process::{exit, Command};

const CGROUP: &str = "/sys/fs/cgroup/minirun";
const MEMORY_LIMIT: &str = "100M";
const PIDS_LIMIT: &str = "20";

// Create our cgroup, set its limits, and move minirun itself into it.
// Every child minirun starts afterwards is born inside the group.
fn create_cgroup() -> std::io::Result<()> {
    fs::create_dir_all(CGROUP)?;
    fs::write(format!("{}/memory.max", CGROUP), MEMORY_LIMIT)?;
    let _ = fs::write(format!("{}/memory.swap.max", CGROUP), "0"); // no swapping, if swap exists
    fs::write(format!("{}/pids.max", CGROUP), PIDS_LIMIT)?;
    fs::write(format!("{}/cgroup.procs", CGROUP), std::process::id().to_string())?;
    Ok(())
}

// Move minirun back to the top-level group, then delete ours.
fn remove_cgroup() {
    let _ = fs::write("/sys/fs/cgroup/cgroup.procs", std::process::id().to_string());
    let _ = fs::remove_dir(CGROUP);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 || args[1] != "run" {
        eprintln!("usage: minirun run <image-folder> <command> [args...]");
        exit(2);
    }

    let rootfs = args[2].clone();
    let program = &args[3];
    let program_args = &args[4..];

    // In the PARENT: our next child will be born into a new PID namespace.
    if let Err(error) = unshare(CloneFlags::CLONE_NEWPID) {
        eprintln!("[minirun] cannot create PID namespace: {} (did you use sudo?)", error);
        exit(1);
    }

    // In the PARENT: create the cgroup and step into it.
    if let Err(error) = create_cgroup() {
        eprintln!("[minirun] cannot set up cgroup: {}", error);
        remove_cgroup();
        exit(1);
    }

    println!("[minirun] parent pid on the host: {}", std::process::id());
    println!("[minirun] image: {}", rootfs);
    println!("[minirun] limits: memory {}, processes {}", MEMORY_LIMIT, PIDS_LIMIT);
    println!("[minirun] starting: {} {:?}", program, program_args);

    let mut cmd = Command::new(program);
    cmd.args(program_args);

    // In the CHILD: after fork, before exec.
    unsafe {
        cmd.pre_exec(move || {
            unshare(CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNS)?;
            sethostname("container")?;
            mount(
                None::<&str>,
                "/",
                None::<&str>,
                MsFlags::MS_REC | MsFlags::MS_PRIVATE,
                None::<&str>,
            )?;
            chroot(rootfs.as_str())?;
            chdir("/")?;
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

    let result = cmd.status();
    remove_cgroup();

    match result {
        Ok(status) => println!("[minirun] finished: {}", status),
        Err(error) => {
            eprintln!("[minirun] could not start '{}': {}", program, error);
            exit(1);
        }
    }
}
