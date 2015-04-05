#![feature(libc)]

extern crate libc;

use std::ffi::CString;

use std::env;
use std::io::Write;
use std::process;

use libc::types::common::c95::c_void;
use libc::funcs::posix88 as posix88_f;
use libc::consts::os::posix88 as posix88_c;
use libc::consts::os::extra;

fn read_print_file(path: &str) {
    let c_path = CString::new(path).unwrap();
    let fd = unsafe {
        posix88_f::fcntl::open(
            c_path.as_ptr(),
            posix88_c::O_RDONLY,
            0)
    };

    let address = unsafe {
        posix88_f::mman::mmap(
            0 as *mut c_void,
            4096,
            posix88_c::PROT_READ,
            posix88_c::MAP_PRIVATE
          | extra::MAP_POPULATE,
            fd,
            0
            )
    };

    let buffer : &[u8] = unsafe {
        std::slice::from_raw_parts(address as *const u8, 4096)
    };

    println!("{:?}", buffer);

    unsafe {
        posix88_f::unistd::close(fd);
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
