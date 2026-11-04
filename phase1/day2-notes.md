Concepts:
	1. A program runs in user mode and must ask the kernel for anything real through a system call.
	2. Path of a syscall: lbc wrapper -> number in rax -> syscall instruction -> kernel syscall table -> handler -> result in rax
	3. A process is a running program: PID, parent PID (PPID), memory map, open file descriptors, state
	4. fd 0 - stdin, fd 1 - stdout, fd 2 - stderr; new files get the lowest free number.
