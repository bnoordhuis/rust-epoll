/*
 * Copyright (c) 2012, Ben Noordhuis <info@bnoordhuis.nl>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

#[license = "ISC"];

#[link(name = "epoll",
       vers = "1.0",
       author = "Ben Noordhuis <info@bnoordhuis.nl>")];

//export EPOLL_NONBLOCK;
export EPOLL_CLOEXEC;
export EPOLL_CTL_ADD;
export EPOLL_CTL_DEL;
export EPOLLIN;
export EPOLLPRI;
export EPOLLOUT;
export EPOLLERR;
export EPOLLHUP;
export EPOLLONESHOT;
export EPOLLET;
export epoll_event;
export epoll_create1;
export epoll_ctl;
export epoll_wait;

use std; // required by the tests

import c_int = ctypes::c_int;

//const EPOLL_NONBLOCK: int = 0x800;
const EPOLL_CLOEXEC: int = 0x80000;

const EPOLL_CTL_ADD: int = 1;
const EPOLL_CTL_DEL: int = 2;
const EPOLL_CTL_MOD: int = 3;

const EPOLLIN: i32 = 0x01i32;
const EPOLLPRI: i32 = 0x02i32;
const EPOLLOUT: i32 = 0x04i32;
const EPOLLERR: i32 = 0x08i32;
const EPOLLHUP: i32 = 0x10i32;
const EPOLLONESHOT: i32 = 0x40000000i32;
const EPOLLET: i32 = 0x80000000i32;

type epoll_event = {
  events: i32,
  data: u64
};

#[nolink]
native mod __glibc {
  fn epoll_create1(flags: c_int) -> c_int;
  /*
  fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: epoll_event) -> c_int;
  fn epoll_wait(epfd: c_int,
                events: *mutable epoll_event,
                maxevents: c_int,
                timeout: c_int) -> c_int;
  */
  fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *u8) -> c_int;
  fn epoll_wait(epfd: c_int,
                events: *mutable u8,
                maxevents: c_int,
                timeout: c_int) -> c_int;
}

fn epoll_create1(flags: int) -> int {
  __glibc::epoll_create1(flags as c_int) as int
}

fn epoll_ctl(epfd: int, op: int, fd: int, event: epoll_event) -> int {
  /*
  __glibc::epoll_ctl(epfd as c_int, op as c_int, fd as c_int, event) as int
  */

  let buf: [mutable u8] = vec::init_elt_mut(12u, 0u8);

  // rust as of 2012-02-06 does not support packed types, hence we have to do
  // the packing and unpacking ourselves
  unsafe {
    let p1: *mutable i32 = unsafe::reinterpret_cast(ptr::mut_addr_of(buf[0]));
    let p2: *mutable u64 = unsafe::reinterpret_cast(ptr::mut_addr_of(buf[4]));
    *p1 = event.events;
    *p2 = event.data;
  }

  ret __glibc::epoll_ctl(epfd as c_int,
                         op as c_int,
                         fd as c_int,
                         ptr::addr_of(buf[0])) as int
}

fn epoll_wait(epfd: int, events: [mutable epoll_event], timeout: int) -> int {
  /*
  let pevents: *mutable epoll_event = ptr::mut_addr_of(events[0]);
  let maxevents: c_int = vec::len(events) as c_int;
  ret __glibc::epoll_wait(epfd as c_int,
                          pevents,
                          maxevents,
                          timeout as c_int) as int;
  */

  let buf: [mutable u8] = vec::init_elt_mut(12u * vec::len(events), 0u8);

  let nevents = __glibc::epoll_wait(epfd as c_int,
                                    ptr::mut_addr_of(buf[0]),
                                    vec::len(events) as c_int,
                                    timeout as c_int) as int;

  if (nevents == -1) {
    ret -1;
  }

  // rust as of 2012-02-06 does not support packed types, hence we have to do
  // the packing and unpacking ourselves
  let i = 0;
  while (i < nevents) {
    unsafe {
      let p1: *i32 = unsafe::reinterpret_cast(ptr::addr_of(buf[i * 12]));
      let p2: *u64 = unsafe::reinterpret_cast(ptr::addr_of(buf[i * 12 + 4]));
      events[i] = {events: *p1, data: *p2};
    }
    i += 1;
  }

  ret nevents;
}

#[test]
fn test_epoll_create1() {
  assert epoll_create1(0) >= 0;
  assert epoll_create1(EPOLL_CLOEXEC) >= 0;
  assert epoll_create1(-1) == -1;
}

#[test]
fn test_epoll_ctl() {
  let epfd = epoll_create1(0);
  assert epfd >= 0;

  assert epoll_ctl(epfd, EPOLL_CTL_ADD, 0, {events:EPOLLIN, data:0u64}) == 0;
  assert epoll_ctl(epfd, EPOLL_CTL_ADD, 0, {events:EPOLLIN, data:0u64}) == -1;
  assert epoll_ctl(epfd, EPOLL_CTL_MOD, 0, {events:EPOLLOUT, data:0u64}) == 0;
  assert epoll_ctl(epfd, EPOLL_CTL_DEL, 0, {events:EPOLLIN, data:0u64}) == 0;

  assert epoll_ctl(epfd, EPOLL_CTL_ADD, -1, {events:EPOLLIN, data:0u64}) == -1;
  assert epoll_ctl(epfd, EPOLL_CTL_MOD, -1, {events:EPOLLIN, data:0u64}) == -1;
  assert epoll_ctl(epfd, EPOLL_CTL_DEL, -1, {events:EPOLLIN, data:0u64}) == -1;
}

#[test]
fn test_epoll_wait() {
  // add stdout to epoll set and wait for it to become writable
  // should be immediate, it's an error if we hit the 50 ms timeout
  let epfd = epoll_create1(0);
  assert epfd >= 0;

  let magic = 42u64;
  assert epoll_ctl(epfd, EPOLL_CTL_ADD, 1, {events:EPOLLOUT, data:magic}) == 0;
  assert epoll_ctl(epfd, EPOLL_CTL_ADD, 2, {events:EPOLLOUT, data:magic}) == 0;

  let events: [mutable epoll_event] = [
    mutable {events:0i32, data:0u64}, {events:0i32, data:0u64}];

  let n = epoll_wait(epfd, events, 50);
  assert n == 2;
  assert events[0].data == magic;
  assert events[0].events & EPOLLOUT == EPOLLOUT;
}
