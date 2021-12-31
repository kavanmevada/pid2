#[macro_export]
macro_rules! errno { () => (*sys!(__errno_location())) }

#[macro_export]
macro_rules! _e { ($($arg:expr)+) => (panic!("ERROR: {}", format!($($arg),+))) }

#[macro_export]
macro_rules! _d { ($($arg:expr)+) => (println!("DEBUG: {}", format!($($arg),+))) }

#[macro_export]
macro_rules! _w { ($($arg:expr)+) => (println!("WARNING: {}", format!($($arg),+))) }

#[macro_export]
macro_rules! _pe { ($($arg:expr)+) => (println!("ERROR: {}: {}", format!($($arg),+), str!(sys!(strerror((*sys!(__errno_location()))))))) }
