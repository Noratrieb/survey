use std::{
    fmt::{Debug, Formatter},
    io,
    io::{Read, Write},
    mem,
    mem::MaybeUninit,
    os::unix,
};

const SOCKADDR_IN_SIZE: libc::socklen_t = mem::size_of::<libc::sockaddr_in>() as _;

macro_rules! check_is_zero {
    ($result:expr) => {
        if $result != 0 {
            return Err(io::Error::last_os_error());
        }
    };
}

pub struct SyncTcpListener {
    fd: unix::io::RawFd,
    addr: libc::sockaddr_in,
}

impl SyncTcpListener {
    pub fn bind_any(port: u16) -> io::Result<Self> {
        let socket = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };

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

    pub fn accept(&mut self) -> impl Iterator<Item = SyncTcpStream> + '_ {
        std::iter::from_fn(|| {
            let mut peer_sockaddr = MaybeUninit::uninit();
            let mut sockaddr_size = 0;
            let fd =
                unsafe { libc::accept(self.fd, peer_sockaddr.as_mut_ptr(), &mut sockaddr_size) };
            if fd == -1 {
                return None;
            }

            let peer_sockaddr = unsafe {
                peer_sockaddr
                    .as_mut_ptr()
                    .cast::<libc::sockaddr_in>()
                    .read()
            };

            Some(SyncTcpStream { fd, peer_sockaddr })
        })
    }
}

impl Drop for SyncTcpListener {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

pub struct SyncTcpStream {
    fd: unix::io::RawFd,
    peer_sockaddr: libc::sockaddr_in,
}

impl Read for SyncTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = unsafe { libc::read(self.fd, buf.as_mut_ptr().cast(), buf.len()) };
        if size == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(size.try_into().unwrap())
    }
}

impl Write for SyncTcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = unsafe { libc::write(self.fd, buf.as_ptr().cast(), buf.len()) };
        if size == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(size.try_into().unwrap())
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl Drop for SyncTcpStream {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

impl Debug for SyncTcpStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncTcpStream")
            .field("fd", &self.fd)
            .field("peer_addr", &format_addr(self.peer_sockaddr.sin_addr))
            .finish()
    }
}
