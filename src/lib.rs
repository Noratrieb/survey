#![warn(rust_2018_idioms)]
#![allow(clippy::single_component_path_imports)] // lol clippy

pub mod epoll;
pub mod sync_tcp;

#[cfg(not(target_os = "linux"))]
compile_error!("yeah not gonna compile that here, rip you");

const SOCKADDR_IN_SIZE: libc::socklen_t = std::mem::size_of::<libc::sockaddr_in>() as _;

fn format_addr(addr: libc::sockaddr_in) -> String {
    let bytes = addr.sin_addr.s_addr.to_be_bytes();
    format!(
        "{}.{}.{}.{}:{}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        u16::from_be_bytes(addr.sin_port.to_ne_bytes())
    )
}

macro_rules! check_is_zero {
    ($result:expr) => {
        if $result != 0 {
            return Err(io::Error::last_os_error());
        }
    };
}

use check_is_zero;

macro_rules! check_non_neg1 {
    ($result:expr) => {{
        let result = $result;
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        result
    }};
}

use check_non_neg1;
