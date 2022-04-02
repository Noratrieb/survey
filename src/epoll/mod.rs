use std::{
    io,
    mem::MaybeUninit,
    os::unix::io::{AsRawFd, RawFd},
    ptr::addr_of,
};

use crate::{check_non_neg1, epoll::tcp::AsyncTcpListener};

mod tcp;

unsafe fn make_nonblock(fd: RawFd) -> io::Result<()> {
    let status = check_non_neg1!(libc::fcntl(fd, libc::F_GETFL));
    check_non_neg1!(libc::fcntl(fd, libc::F_SETFL, status | libc::O_NONBLOCK));
    Ok(())
}

pub fn example_from_man_page() -> io::Result<()> {
    let listener = AsyncTcpListener::bind_any(8888)?;
    println!("Created listener {listener:?}");

    unsafe {
        let listen_sock = listener.as_raw_fd();

        let epollfd = check_non_neg1!(libc::epoll_create1(0));

        println!("Created epoll instance");

        let mut ev = libc::epoll_event {
            events: libc::EPOLLIN as _,
            u64: listen_sock as _,
        };

        check_non_neg1!(libc::epoll_ctl(
            epollfd,
            libc::EPOLL_CTL_ADD,
            listen_sock,
            &mut ev
        ));

        loop {
            let mut events = [libc::epoll_event { events: 0, u64: 0 }; 16];

            let nfds = check_non_neg1!(libc::epoll_wait(
                epollfd,
                events.as_mut_ptr(),
                events.len() as _,
                -1,
            ));

            for event in &events[0..nfds as _] {
                if event.u64 == listen_sock as _ {
                    // our TCP listener received a new connection
                    let mut peer_sockaddr = MaybeUninit::uninit();
                    let mut sockaddr_size = 0;

                    let conn_sock = check_non_neg1!(libc::accept(
                        listener.as_raw_fd(),
                        peer_sockaddr.as_mut_ptr(),
                        &mut sockaddr_size,
                    ));

                    make_nonblock(conn_sock)?;
                    let mut ev = libc::epoll_event {
                        events: (libc::EPOLLIN | libc::EPOLLET) as _,
                        u64: conn_sock as _,
                    };
                    check_non_neg1!(libc::epoll_ctl(
                        epollfd,
                        libc::EPOLL_CTL_ADD,
                        conn_sock,
                        &mut ev
                    ));
                    println!("Received new connection! (fd: {conn_sock})");
                } else {
                    println!(
                        "something else happened! {}",
                        addr_of!(event.u64).read_unaligned()
                    );
                }
            }
        }
    }
}
