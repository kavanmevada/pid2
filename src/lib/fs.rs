use crate::{_d, _pe};

pub(crate) fn fs_init() {
    struct Fs<'a>{ spec: &'a str, file: &'a str, fstype: &'a str }

    let filesystems = [
        Fs { spec: "proc",     file: "/proc", fstype: "proc"     },
        Fs { spec: "devtmpfs", file: "/dev",  fstype: "devtmpfs" },
        Fs { spec: "sysfs",    file: "/sys",  fstype: "sysfs"    }
    ];

    for fs in filesystems {
        /*
		 * Check if already mounted, we may be running in a
		 * container, or an initramfs ran before us.  The
		 * function fismnt() reliles on /proc/mounts being
		 * unique for each chroot/container.
		 */
		if fismnt(fs.file) != 0 {
            continue;
        }

        fs_mount(fs.spec, fs.file, fs.fstype, 0, core::ptr::null());
    }
}


pub(crate) fn fs_mount(src: &str, tgt: &str, fstype: &str, flags: u64, data: *const libc::c_void) {
    let msg = if !fstype.is_empty() {
        "MS_MOVE"
    } else {
        "mounting"
    };

    let rc = sys!(mount(c_str!(src), c_str!(tgt), c_str!(fstype), flags, data));
    if rc != 0 && ERRNO!() != libc::EBUSY {
        _pe!("Failed {} {} on {}" msg src tgt);
    } else {
		_d!("Successfuly mounted {} on {}" src tgt);
	}
}

fn ismnt(file: &str, dir: &str, mode: &str) -> i32 {

    let mut found = 0;

    let fp = sys!(setmntent(c_str!(file), c_str!("r")));
    if fp.is_null() {
        return 0; /* Dunno, maybe not */
    }

    let mut mnt;
    loop {
        mnt = sys!(getmntent(fp));
        if mnt.is_null() {
            break;
        };

        if str!((*mnt).mnt_dir) == dir {
            if !mode.is_empty() {
                if hasopt(str!((*mnt).mnt_opts), mode) {
                    found = 1;
                }
            } else {
                found = 1;
            }

            break;
        }
    }

    sys!(endmntent(fp));

    return found;
}

fn hasopt(opts: &str, opt: &str) -> bool {
    !opts.split(',').all(|a| a != opt)
}

/* Requires /proc to be mounted */
pub(crate) fn fismnt(dir: &str) -> i32 {
    ismnt("/proc/mounts", dir, "")
}