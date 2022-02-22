#[macro_export]
macro_rules! sys {
    ($fn:ident $args:tt) => (unsafe { use libc::$fn; $fn $args });
}

#[macro_export]
macro_rules! ERRNO {
    () => {
        unsafe { *sys!(__errno_location()) }
    };
}

#[macro_export]
macro_rules! c_str {
    ($str:expr) => {
        [$str.as_bytes(), &[0u8; 1]].concat().as_slice().as_ptr() as *const _ as *const i8
    };
}

#[macro_export]
macro_rules! strlen {
    ($ptr:expr) => {{
        let mut p = $ptr as *const i8;
        while p.as_ref().unwrap_or(&0) != &0 {
            p = p.add(1)
        }
        p.offset_from($ptr) as usize
    }};
}

#[macro_export]
macro_rules! str {
    ($ptr:expr) => {
        unsafe {
            core::str::from_utf8(core::slice::from_raw_parts(
                $ptr as *const u8,
                strlen!($ptr),
            ))
        }
        .expect("Non UTF-8 char detected in str")
    };
}


pub(crate) mod log;
pub(crate) mod utils;



pub(crate) mod fs;

pub(crate) mod exec;
pub(crate) use exec::run;

pub(crate) mod uev;

// #[macro_export]
// macro_rules! str {
//     (u $ptr:expr) => {
//         str!($ptr, u8)
//     };
//     (i $ptr:expr) => {
//         str!($ptr, i8)
//     };
//     ($ptr:expr, $t:ty) => {
//         unsafe {
//             core::str::from_utf8(core::slice::from_raw_parts($ptr as *const u8, {
//                 let mut p = $ptr as *const $t;
//                 while p.as_ref().unwrap_or(&0) != &0 {
//                     p = p.add(1)
//                 }
//                 p.offset_from($ptr) as usize
//             }))
//         }
//         .expect("Non UTF-8 char detected in str")
//     };
// }

// #[macro_export]
// macro_rules! bit {
//     ($res:expr) => {
//         $res != 0
//     };
// }

// #[macro_export]
// macro_rules! ARRAY(
//     [$($index:expr)+] => ([$(if $index.is_empty() { core::ptr::null() } else { str!(c $index) },)+]);
// );
