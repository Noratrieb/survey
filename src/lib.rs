#![warn(rust_2018_idioms)]
#![allow(dead_code)]

pub mod epoll;
pub mod listener;

#[cfg(not(target_os = "linux"))]
compile_error!("yeah not gonna compile that here, rip you");
