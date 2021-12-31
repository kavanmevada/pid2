use crate::_d;

/* * Event loop context, need one per process and thread */
pub type uev_ctx_t<const N: usize> = uev_ctx<N>;

pub struct uev_ctx<const N: usize> {
    pub running: bool,
    pub fd: i32,
    pub maxevents: usize,
    pub watchers: [uev; N],
    pub workaround: u32,
}

impl<const N: usize> Default for uev_ctx_t<N> {
    fn default() -> Self {
        Self {
			running: false,
			fd: 0i32,
			maxevents: 0usize,
			watchers: [uev::default(); N],
			workaround: 0u32,
        }
    }
}

pub type uev_t = uev;

#[derive(Clone, Copy)]
pub struct uev {
	pub r#type: uev_type_t,
    // pub type_0: i32,
    // pub signo: i32,
    pub fd: i32,
    // pub ctx: [uev_ctx_t<N>; N],
    // pub siginfo: libc::signalfd_siginfo,
	events: i32
}

impl Default for uev {
    fn default() -> Self {
        Self {
            r#type: uev_type_t::UEV_EVENT_TYPE,
            fd: 0,
			events: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
/* I/O, timer, or signal watcher */
pub enum uev_type_t {
	UEV_IO_TYPE = 1,
	UEV_SIGNAL_TYPE,
	UEV_TIMER_TYPE,
	UEV_CRON_TYPE,
	UEV_EVENT_TYPE,
}

const UEV_MAX_EVENTS: usize	= 10;		/**< Max. number of simulateneous events */

/* I/O events, signal and timer revents are always UEV_READ */
const UEV_NONE: i32			= 0i32;		/**< normal loop      */
const UEV_ERROR: i32		= libc::EPOLLERR;	/**< error flag       */
const UEV_READ: i32			= libc::EPOLLIN;		/**< poll for reading */
const UEV_WRITE: i32		= libc::EPOLLOUT;	/**< poll for writing */
const UEV_PRI: i32			= libc::EPOLLPRI;	/**< priority message */
const UEV_HUP: i32			= libc::EPOLLHUP;	/**< hangup event     */
const UEV_RDHUP: i32		= libc::EPOLLRDHUP;	/**< peer shutdown    */
const UEV_EDGE: i32			= libc::EPOLLET;		/**< edge triggered   */
const UEV_ONESHOT: i32		= libc::EPOLLONESHOT;	/**< one-shot event   */

/* Run flags */
const UEV_ONCE: i32		= 1;		/**< run loop once    */
const UEV_NONBLOCK: i32		= 2;		/**< exit if no event */

const UEV_EVENT_MASK: i32 = UEV_ERROR | UEV_READ | UEV_WRITE | UEV_PRI | UEV_RDHUP | UEV_HUP  | UEV_EDGE  | UEV_ONESHOT;


fn _init<const N: usize>(ctx: &mut uev_ctx_t<N>, close_old: i32) -> i32 {

	let fd = sys!(epoll_create1(libc::EPOLL_CLOEXEC));
	if fd < 0 {
		return -1;
	}

	if close_old != 0 {
		sys!(close(ctx.fd));
	}

	ctx.fd = fd;

	return 0;
}


fn has_data(fd: i32) -> bool {
	// struct timeval timeout = { 0, 0 };
	let mut fds: libc::fd_set = unsafe { core::mem::zeroed() };
	let n = 0;

	sys!(FD_ZERO(&mut fds));
	sys!(FD_SET(fd, &mut fds));

	if (sys!(select(1, &mut fds, core::ptr::null_mut(), core::ptr::null_mut(), &mut libc::timeval { tv_sec: 0, tv_usec: 0 })) > 0) {
		return sys!(ioctl(0, libc::FIONREAD, &n)) == 0 && n > 0;
	}

	drop(fds);

	return false;
}


fn uev_init1<const N: usize>(ctx: &mut uev_ctx_t<N>, maxevents: usize) -> i32 {
	if maxevents < 1 {
		println!("EINVAL error!");
		return -1;
	}

	// memset(ctx, 0, sizeof(*ctx));
	ctx.maxevents = maxevents;

	return _init(ctx, 0);
}

pub fn uev_init<const N: usize>(ctx: &mut uev_ctx_t<N>) -> i32 {
	return uev_init1(ctx, UEV_MAX_EVENTS);
}

fn uev_io_init<const N: usize>(ctx: &mut uev_ctx_t<N>, cb: Box<dyn Fn(i32)>, fd: i32, events: i32) -> i32 {
	if fd < 0 {
		println!("EINVAL error!");
		return -1;
	}

	// struct epoll_event ev;

	if _uev_watcher_active(w) {
		return 0;
	}

	let w = uev_t {
		r#type: uev_type_t::UEV_IO_TYPE,
		fd,
		events: events,
	};
		

	let ev = libc::epoll_event {
		events:	w.events | libc::EPOLLRDHUP,
		u64:	w,
	};


	if (epoll_ctl(ctx.fd, libc::EPOLL_CTL_ADD, fd, &ev) < 0) {
		if (ERRNO!() != libc::EPERM) {
			return -1;
		}
			

		/* Handle special case: `application < file.txt` */
		if w.r#type != uev_type_t::UEV_IO_TYPE || w.events != UEV_READ {
			return -1;
		}

		/* Only allow this special handling for stdin */
		if w.fd != libc::STDIN_FILENO {
			return -1;
		}
			

		ctx.workaround = 1;
		w.active = false;
	} else {
		w.active = 1;
	}

	/* Add to internal list for bookkeeping */
	_UEV_INSERT(w, ctx.watchers);


	// if _uev_watcher_init(ctx, w, uev_type_t::UEV_IO_TYPE, cb, fd, events) {
	// 	return -1;
	// }

	// return _uev_watcher_start(w);
}




pub fn uev_run<const N: usize>(ctx: &mut uev_ctx_t<N>, flags: i32) -> i32 {
	let mut timeout = -1;

	if (flags & UEV_NONBLOCK) != 0 { timeout = 0 }

	/* Start the event loop */
	ctx.running = true;

	while ctx.running /* TODO && ctx.watchers */ {
		_d!("Loop ctx.running");
		let mut ee = [libc::epoll_event {
				events: 0,
				u64: 0,
			}; UEV_MAX_EVENTS];

		let i = 0;
		let nfds = 0;
		let rerun = false;

		if rerun { continue }
		ctx.workaround = 0;

		loop {
			_d!("Loop epoll_wait");
			let nfds = sys!(epoll_wait(ctx.fd, ee.as_mut_ptr(), ctx.maxevents as i32, timeout));
			if nfds > 0 || !ctx.running { break }

			if libc::EINTR == ERRNO!() { continue; /* Signalled, try again */ }

			/* Unrecoverable error, cleanup and exit with error. */
			uev_exit(ctx);

			return -2;
		}

		// ctx.running

		for i in 0..nfds {
			let w = ee[i].u64 as *mut uev_t;
			if let Some(w) = core::ptr::NonNull::new(w) {
				let w = unsafe { w.as_ref() };
				let events = ee[i].events;


				match w.r#type {
					uev_type_t::UEV_IO_TYPE => {
						if (events as i32 & (libc::EPOLLHUP | libc::EPOLLERR)) != 0 {
							uev_io_stop(w);
						}
					},
					uev_type_t::UEV_SIGNAL_TYPE => todo!(),
					uev_type_t::UEV_TIMER_TYPE => todo!(),
					uev_type_t::UEV_CRON_TYPE => todo!(),
					uev_type_t::UEV_EVENT_TYPE => todo!(),
				}


			}

		}
	}

	return  -1;
}


fn uev_exit<const N: usize>(ctx: &mut uev_ctx_t<N>) {
	println!("uev_exit");
}

fn uev_io_stop(ctx: &uev_t) {
	println!("uev_io_stop");
}