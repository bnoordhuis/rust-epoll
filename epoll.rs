// Copyright (c) 2012, Ben Noordhuis <info@bnoordhuis.nl>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

#![feature(custom_attribute)]
#![license = "ISC"]

#![link(name = "epoll",
        vers = "0.0.2",
        author = "Ben Noordhuis <info@bnoordhuis.nl>")]

//pub const EPOLL_NONBLOCK: u32 = 0x800;
pub const EPOLL_CLOEXEC: u32 = 0x80000;

pub const EPOLL_CTL_ADD: u32 = 1;
pub const EPOLL_CTL_DEL: u32 = 2;
pub const EPOLL_CTL_MOD: u32 = 3;

pub const EPOLLIN: u32 = 0x01;
pub const EPOLLPRI: u32 = 0x02;
pub const EPOLLOUT: u32 = 0x04;
pub const EPOLLERR: u32 = 0x08;
pub const EPOLLHUP: u32 = 0x10;
pub const EPOLLONESHOT: u32 = 0x40000000;
pub const EPOLLET: u32 = 0x80000000;

#[cfg(target_arch = "x86_64")]
#[repr(C, packed)]
pub struct epoll_event {
  events: u32,
  data: u64
}

#[cfg(not(target_arch = "x86_64"))]
#[repr(C)]
pub struct epoll_event {
  events: i32,
  data: u64
}

mod __glibc {
  use epoll_event;

  extern {
    pub fn epoll_create1(flags: u32) -> i32;
    pub fn epoll_ctl(epfd: i32, op: u32, fd: i32,
                     event: *const epoll_event) -> i32;
    pub fn epoll_wait(epfd: i32, events: *mut epoll_event, maxevents: i32,
                      timeout: i32) -> i32;
  }
}

pub fn epoll_create() -> i32 {
  unsafe { __glibc::epoll_create1(0) }
}

pub fn epoll_create1(flags: u32) -> i32 {
  unsafe { __glibc::epoll_create1(flags) }
}

pub fn epoll_ctl(epfd: i32, op: u32, fd: i32,
                 event: &epoll_event) -> i32 {
  unsafe { __glibc::epoll_ctl(epfd, op, fd, event) }
}

pub fn epoll_wait(epfd: i32, events: &mut [epoll_event],
                  timeout: i32) -> i32 {
  unsafe { __glibc::epoll_wait(epfd, events.as_mut_ptr(),
                               events.len() as i32, timeout) }
}

#[test]
fn test_epoll_create1() {
  assert!(epoll_create1(0) >= 0);
  assert!(epoll_create1(EPOLL_CLOEXEC) >= 0);
  assert!(epoll_create1(-1) == -1);
}

#[test]
fn test_epoll_ctl() {
  let epfd = epoll_create1(0);
  assert!(epfd >= 0);
  assert!(epoll_ctl(epfd, EPOLL_CTL_ADD, 0,
                    &epoll_event { events: EPOLLIN, data: 0 }) == 0);
  assert!(epoll_ctl(epfd, EPOLL_CTL_ADD, 0,
                    &epoll_event { events: EPOLLIN, data: 0}) == -1);
  assert!(epoll_ctl(epfd, EPOLL_CTL_MOD, 0,
                    &epoll_event { events: EPOLLOUT, data: 0}) == 0);
  assert!(epoll_ctl(epfd, EPOLL_CTL_DEL, 0,
                    &epoll_event { events: EPOLLIN, data: 0}) == 0);
  assert!(epoll_ctl(epfd, EPOLL_CTL_ADD, -1,
                    &epoll_event { events: EPOLLIN, data: 0}) == -1);
  assert!(epoll_ctl(epfd, EPOLL_CTL_MOD, -1,
                    &epoll_event { events: EPOLLIN, data: 0}) == -1);
  assert!(epoll_ctl(epfd, EPOLL_CTL_DEL, -1,
                    &epoll_event { events: EPOLLIN, data: 0}) == -1);
}

#[test]
fn test_epoll_wait() {
  // add stdout to epoll set and wait for it to become writable
  // should be immediate, it's an error if we hit the 50 ms timeout
  let epfd = epoll_create1(0);
  assert!(epfd >= 0);

  let magic = 42;
  let event = epoll_event { events: EPOLLOUT, data: magic };
  assert!(epoll_ctl(epfd, EPOLL_CTL_ADD, 1, &event) == 0);
  assert!(epoll_ctl(epfd, EPOLL_CTL_ADD, 2, &event) == 0);

  let mut events = [epoll_event {events: 0, data: 0},
                    epoll_event {events: 0, data: 0}];
  let n = epoll_wait(epfd, &mut events, 50);
  assert!(n == 2);
  assert!(events[0].data == magic);
  assert!(events[0].events & EPOLLOUT == EPOLLOUT);
}
