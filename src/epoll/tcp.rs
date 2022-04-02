use std::{
    fmt::{Debug, Formatter},
    io,
    os::{unix, unix::io::RawFd},
};

use crate::{check_is_zero, format_addr, SOCKADDR_IN_SIZE};

pub struct AsyncTcpListener {
    fd: unix::io::RawFd,
    addr: libc::sockaddr_in,
}

impl AsyncTcpListener {
    pub fn bind_any(port: u16) -> io::Result<Self> {
        let socket =
            unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_NONBLOCK, 0) };

        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        let addr = libc::sockaddr_in {
            sin_family: libc::AF_INET.try_into().unwrap(),
            sin_port: port.to_be(),
            sin_addr: libc::in_addr {
                s_addr: libc::INADDR_ANY,
            },
            sin_zero: [0; 8],
        };
        let addr_erased_ptr = &addr as *const libc::sockaddr_in as _;

        let result = unsafe { libc::bind(socket, addr_erased_ptr, SOCKADDR_IN_SIZE) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        check_is_zero!(unsafe { libc::listen(socket, 5) });

        Ok(Self { fd: socket, addr })
    }
}

impl unix::io::AsRawFd for AsyncTcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for AsyncTcpListener {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

impl Debug for AsyncTcpListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncTcpListener")
            .field("fd", &self.fd)
            .field("addr", &format_addr(self.addr))
            .finish()
    }
}
