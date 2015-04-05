#![feature(libc)]

extern crate libc;

use std::ffi::CString;

use std::env;
use std::io::Write;
use std::process;

use libc::types::common::c95::c_void;

fn read_print_file(path: &str) {
    let cstr = CString::new(path).unwrap();
    let fd = unsafe {
        libc::funcs::posix88::fcntl::open(
            cstr.as_ptr(),
            libc::consts::os::posix88::O_RDONLY,
            0)
    };

    let address = unsafe {
        libc::funcs::posix88::mman::mmap(
            0 as *mut c_void,
            4096,
            libc::consts::os::posix88::PROT_READ,
            libc::consts::os::posix88::MAP_PRIVATE
          | libc::consts::os::extra::MAP_POPULATE,
            fd,
            0
            )
    };

    let buffer : &[u8] = unsafe {
        std::slice::from_raw_parts(address as *const u8, 4096)
    };

    println!("{:?}", buffer);

    unsafe {
        libc::funcs::posix88::unistd::close(fd);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut stderr = std::io::stderr();
    if args.len() != 2 {
        writeln!(&mut stderr, "Usage: rexdump <file>").unwrap();
        process::exit(1);
    }
    read_print_file(&args[1]);
}
