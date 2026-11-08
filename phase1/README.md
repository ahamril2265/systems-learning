# Phase 1: Linux fundamentals, practical run

Hands-on run on Ubuntu 26.04.1 LTS (VirtualBox VM, 6 vCPU, 8 GB).

## Processes
- Created a zombie (`STAT Z`) by having a parent `exec` into a program that never calls `wait`.
- Orphaned a process: it was adopted by `systemd --user` (PID 3381), a child subreaper, not PID 1.

## Syscalls and file descriptors
- `strace ls`: 149 syscalls for one directory listing; most were the dynamic loader (`mmap`, `openat`).
- `cat /etc/shadow` failed at `openat` with `-1 EACCES`; file is `-rw-r----- root shadow`.
- `> out.txt` only redirects fd 1; errors on fd 2 still reach the screen until `2>&1`.

## Signals
- SIGTERM can be caught (trap ran cleanup) or ignored; SIGKILL cannot.

## Memory
- `mmap` of 500 MB raised VSZ immediately; RSS only grew as pages were touched (~128,000 minor faults).

## Scheduling
- Two CPU-bound loops on one core: nice 0 got 90.1%, nice 10 got 9.6%, matching kernel weights 1024 vs 110.
- `yes` had 3 voluntary vs 1,807 involuntary context switches.

## Filesystems
- Hard link shares the inode (link count 2) and survives deleting the original; symlink (8 bytes) dangles.
- Deleting a file needs write permission on the directory, not the file.

## Networking
- VirtualBox NAT: 10.0.2.15/24 via gateway 10.0.2.2; DNS through systemd-resolved at 127.0.0.53.
- Listening socket was fd 3; connecting to a closed port gave `Connection refused` (RST).

## Namespaces and cgroups
- `unshare --pid --uts`: shell saw itself as PID 1 with hostname `box`; host unchanged.
- `docker run --memory=20m` running `tail /dev/zero` exited 137 = 128 + 9 (SIGKILL by the OOM killer).
