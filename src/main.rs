mod lib;

extern crate alloc;

use std::ffi::c_void;

use std::mem::MaybeUninit;


use std::ptr::NonNull;


use lib::*;





const PATH_SYS_BLOCK: &str  = "/sys/block";
const PATH_SYS_DEVBLOCK: &str   = "/sys/dev/block";

const _PATH_STDPATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";


struct UEvent {
    signo: i32,
	fd: i32,
    cb: Box<dyn Fn(i32)>,
    events: u32,
    active: bool,
}

fn uev_cb(id: i32) {
    dbg!(id);
}

fn main() {
    let mut ctx = uev::uev_ctx::<10>::default();

    uev::uev_init(&mut ctx);

    // uev_timer_init(&ctx, &timerw, timeout, NULL, 100, 2000);


    uev::uev_run(&mut ctx, 0);
}

fn main3() {
    let fd = sys!(epoll_create1(libc::EPOLL_CLOEXEC));
    let inotify_fd = sys!(inotify_init1(libc::IN_CLOEXEC));

    let path = "/tmp/";

    let uevent = Box::new(UEvent {
        signo: 67,
        fd: 34,
        cb: Box::from(uev_cb),
        events: 7,
        active: true,
    });
    let ptr = Box::into_raw(uevent);
    println!("created ptr: {:?}", ptr);

    let mut ev = libc::epoll_event {
        events: (libc::EPOLLIN | libc::EPOLLRDHUP) as u32,
        u64: ptr as *mut _ as u64,
    };

    if sys!(inotify_add_watch(inotify_fd, c_str!(path), libc::IN_CREATE | libc::IN_DELETE)) < 0 {
        panic!("Yo, not working!");
    }

	sys!(epoll_ctl(
        fd,
        libc::EPOLL_CTL_ADD,
        inotify_fd,
        &mut ev
    ));


    if let Some(event) = wait(fd) {
        dbg!((event.cb)(event.fd));
    }
}


fn wait<'a>(fd: i32) -> Option<&'a UEvent> {
    let mut events = [libc::epoll_event {
        events: 0,
        u64: 0,
    }; 10];

    sys!(epoll_wait(fd, &mut events[0], 10, -1));

    let ret_ptr = events[0].u64 as *mut UEvent;
    if let Some(ptr) = NonNull::new(ret_ptr) {
        return Some(unsafe { ptr.as_ref() });
    }

    return None;
}


fn uev_add_watcher(fd: i32, path: &str) -> i32 {
    sys!(inotify_add_watch(fd, c_str!(path), libc::IN_CREATE | libc::IN_DELETE))
}


fn main2() {
    if sys!(getpid()) != 1 {
        _w!("Running as PID: {}" sys!(getpid()));
    }

    // TODO console_init

    /*
	 * Set PATH, SHELL, and PWD early to something sane
	 */
	sys!(setenv(c_str!("PATH"), c_str!(_PATH_STDPATH), 1));
	sys!(setenv(c_str!("SHELL"), c_str!("/bin/sh"), 1));
	sys!(setenv(c_str!("LOGNAME"), c_str!("root"), 1));
	sys!(setenv(c_str!("USER"), c_str!("root"), 1));


    if sys!(chdir(c_str!("/"))) != 0 {
        _pe!("Failed cd /");
    }


    // let cstr = cstr!("Apple");
    // let str = str!(cstr);

    println!("We love 🦀 {}", "🦉");
    println!("UID: {}", sys!(getuid()));
    println!("PID: {}", sys!(getpid()));



    fs::fs_init();
    cgroup_init();

    // if fs::fismnt("/dev/shm") == 0 {
    //     sys!(mkdir(c_str!("/dev/shm"), 777));
    //     fs::fs_mount(
    //         "shm",
    //         "/dev/shm",
    //         "tmpfs",
    //         0,
    //         c_str!("mode=0777") as *const c_void,
    //     );
    // }

    run(&["/sbin/uname", "uname", "-a"]);
    run(&["/bin/ls", "ls", "/sys/fs/cgroup"]);






    // dbg!(fd);


    // let cmd = ARRAY!["/bin/sleep" "sleep" "10" ""];
    // unsafe { libc::execv(cmd[0], cmd[1..].as_ptr()) };

    // unsafe {
    //     let cmd = &["/bin/sleep", "sleep", "10"];
    //     libc::execv(
    //         Box::from_iter(cmd[0].bytes().chain(core::iter::once(0)).map(|c| c as i8)).as_ptr(),
    //         Box::from_iter(Box::from_iter(cmd.iter().map(|&s| Box::from_iter(s.bytes().chain(core::iter::once(0)).map(|c| c as i8))))
    //             .iter()
    //             .skip(1)
    //             .map(|s| s.as_ptr())
    //             .chain(core::iter::once(core::ptr::null()))).as_ptr(),
    //     );
    // }
}







fn cgroup_init() {
	let opts = libc::MS_NODEV | libc::MS_NOEXEC | libc::MS_NOSUID;
    const FINIT_CGPATH: &str = "/sys/fs/cgroup";

    if sys!(mount(c_str!("none"), c_str!(FINIT_CGPATH), c_str!("cgroup2"), opts, core::ptr::null())) != 0 {
		_pe!("Failed mounting cgroup v2");
		return;
	}

    sys!(sleep(2000));

    /* Find available controllers */
	let fp = sys!(open(c_str!([FINIT_CGPATH, "/cgroup.controllers"].concat()), libc::O_RDONLY));
	if fp != 0 {
		_pe!("Failed opening {}" format!("{}{}", FINIT_CGPATH, "/cgroup.controllers"));
		return;
	}


    dbg!(fp);
}

// kmod"/app/ip_tables.ko");
fn kmod(module: &str) {
    let fd = { 
        let _fd = sys!(open(c_str!(module), libc::O_RDONLY));
        if _fd <= 0 {
            _pe!("could not load module {}" module);
            _fd as i64
        } else {
            let mut stat = MaybeUninit::<libc::stat>::uninit();
            sys!(fstat(_fd, stat.as_mut_ptr()));
            let stat = unsafe { stat.assume_init() };
            let stat_size = stat.st_size as usize;
            let mut image = vec![0u8; stat_size];
            sys!(read(_fd, image.as_mut_ptr() as *mut c_void, stat_size));
            let ret = sys!(syscall(libc::SYS_init_module, image.as_mut_ptr() as *mut c_void, stat_size));
            dbg!(ret);
            drop(stat);
            if ret != 0 {
                _pe!("could not load module {}" module);
            }

            ret
        }
    };
}
