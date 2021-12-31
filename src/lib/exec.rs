use crate::sys;
use crate::{_d, _e, _pe, c_str};

/* Wait for process completion, returns status of waitpid(2) syscall */
fn complete(cmd: &str, pid: i32) -> i32 {
    let mut status = 0;

    if (sys!(waitpid(pid, &mut status, 0i32)) == -1) {
        match ERRNO!() {
            libc::EINTR => _e!("Caught unblocked signal waiting for {}, aborting" cmd),
            libc::ECHILD => _e!("Caught SIGCHLD waiting for {}, aborting" cmd),
            _ => _pe!("Failed starting {}" cmd),
        }

        return -1;
    }

    return status;
}

pub(crate) fn run(args: &[&str]) -> i32 {
    let pid = sys!(fork());

    if 0 == pid {
        sys!(setsid());

        /* Always redirect stdio for run() */
        let path = c_str!("/dev/console");
        let fd: i32 = sys!(open(path, libc::O_RDWR));
        drop(path);

        if fd != 0 {
            sys!(dup2(fd, libc::STDIN_FILENO));
            sys!(dup2(fd, libc::STDOUT_FILENO));
            sys!(dup2(fd, libc::STDERR_FILENO));
        }

        // sig_unblock();
        sys!(execv(
            Box::from_iter(args[0].bytes().chain(core::iter::once(0)).map(|c| c as i8)).as_ptr(),
            Box::from_iter(
                Box::from_iter(args.iter().map(|&s| {
                    Box::from_iter(s.bytes().chain(core::iter::once(0)).map(|c| c as i8))
                }))
                .iter()
                .skip(1)
                .map(|s| s.as_ptr())
                .chain(core::iter::once(core::ptr::null())),
            )
            .as_ptr() as *const _,
        ));

        sys!(exit(1)) /* Only if execv() fails. */
    } else if -1 == pid {
        _pe!("{}" args[0]);
        // free(backup);

        return -1;
    }

    let status = complete(args[0], pid);
    if -1 == status {
        //free(backup);
        return 1;
    }

    let mut result = sys!(WEXITSTATUS(status));

    if sys!(WIFEXITED(status)) {
        _d!("Started {} and ended OK: {}" args[0] result); //dbg
    } else if sys!(WIFSIGNALED(status)) {
        _d!(
            "Process {} terminated by signal {}"
            args[0]
            sys!(WTERMSIG(status))
        );
        if result != 0 {
            /* Must alert callee that the command did complete successfully.
             * This is necessary since not all programs trap signals and
             * change their return code accordingly. --Jocke */
            result = 1;
        }
    }

    return result;
}
