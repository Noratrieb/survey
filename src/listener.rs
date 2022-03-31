use std::mem::MaybeUninit;
use std::{io, mem};

const PORT: libc::in_port_t = 1112;

const SOCKADDR_IN_SIZE: libc::socklen_t = mem::size_of::<libc::sockaddr_in>() as _;

macro_rules! check_zero {
    ($result:expr) => {
        if $result != 0 {
            return Err(io::Error::last_os_error());
        }
    };
}

pub fn listener() -> io::Result<()> {
    unsafe {
        let socket = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);

        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        println!("Created socket ({})", socket);

        let addr = libc::sockaddr_in {
            sin_family: libc::AF_INET.try_into().unwrap(),
            sin_port: PORT.to_be(),
            sin_addr: libc::in_addr {
                s_addr: libc::INADDR_ANY,
            },
            sin_zero: [0; 8],
        };
        let addr_erased_ptr = &addr as *const libc::sockaddr_in as _;

        let result = libc::bind(socket, addr_erased_ptr, SOCKADDR_IN_SIZE);
        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        println!("Bound socket ({socket}) on port {PORT}");

        check_zero!(libc::listen(socket, 5));

        println!("Listening on socket ({socket})");

        let mut peer_sockaddr = MaybeUninit::uninit();
        let mut sockaddr_size = 0;
        let connection = libc::accept(socket, peer_sockaddr.as_mut_ptr(), &mut sockaddr_size);
        if connection == -1 {
            return Err(io::Error::last_os_error());
        }

        println!("Received connection! (connfd={connection})");

        check_zero!(libc::close(connection));
        check_zero!(libc::close(socket));
    }

    Ok(())
}
