#![feature(core, libc, page_size)]

extern crate libc;

use std::ffi::CString;

use std::env;
use std::io::{self, Write, Stderr};
use std::process;

use libc::types::common::c95::c_void;
use libc::funcs::posix88 as posix88_f;
use libc::consts::os::posix88 as posix88_c;
use libc::consts::os::extra;
use libc::funcs::c95::stdio;

fn print_offset(offset: i64) {

}

fn print_hex(buffer: &[u8], buffer_size: i64, line_width: i64) {

}

fn align_delimiter(line_size: i64, line_width_elements: i64) {

}

fn print_chars(buffer: &[u8], line_size: i64) {

}

fn print_contents(buffer: &[u8], buffer_size: i64, offset: i64) {
    if buffer_size == 0 {
        return;
    };

    let line_width_elements = 16;
    let mut remaining_buffer_size = buffer_size;
    let mut line_number = 0;
    let mut current_offset = offset;

    for line in buffer.chunks(line_width_elements as usize) {
        let line_size = if remaining_buffer_size > line_width_elements {
            line_width_elements
        } else {
            remaining_buffer_size
        };
        if line_size == 0 {
            break;
        }

        print_offset(line_number * line_width_elements + current_offset);

        print_hex(
            line,
            line_size,
            line_width_elements);

        align_delimiter(line_size, line_width_elements);

        print_chars (line, line_size);

        println!("|");

        line_number += 1;
        current_offset += line_size;
        remaining_buffer_size -= line_size;
    }
}

fn read_print_file(path: &str) -> Result<(), ()> {
    let c_path = CString::new(path).unwrap();
    let fd = unsafe {
        posix88_f::fcntl::open(
            c_path.as_ptr(),
            posix88_c::O_RDONLY,
            0)
    };
    if fd == -1 {
        let c_error = CString::new("Couldn't open file").unwrap();
        unsafe {
            stdio::perror(c_error.as_ptr());
        }
        return Err(());
    }
    let mut file_info : libc::types::os::arch::posix01::stat = unsafe {
        std::mem::uninitialized()
    };
    let result = unsafe {
        posix88_f::stat_::fstat(fd, & mut file_info)
    };
    if result == -1 {
        let c_error = CString::new("Couldn't get file into").unwrap();
        unsafe {
            stdio::perror(c_error.as_ptr());
        }
        return Err(());
    }
    let mut remaining_file_size = file_info.st_size;
    let page_size : i64 = std::num::cast(std::env::page_size()).unwrap();
    let mut offset = 0;
    while remaining_file_size > 0 {
        let map_size: u64 = std::num::cast(
            if remaining_file_size > page_size {
                page_size
            } else {
                remaining_file_size
            }).unwrap();
        let address = unsafe {
            posix88_f::mman::mmap(
                0 as *mut c_void,
                map_size,
                posix88_c::PROT_READ,
                posix88_c::MAP_PRIVATE
              | extra::MAP_POPULATE,
                fd,
                offset)
        };
        if address == posix88_c::MAP_FAILED {
            let c_error = CString::new("Couldn't read file").unwrap();
            unsafe {
                stdio::perror(c_error.as_ptr());
            }
            return Err(());
        };

        let buffer : &[u8] = unsafe {
            std::slice::from_raw_parts(address as *const u8, 4096)
        };

        print_contents(buffer, std::num::cast(map_size).unwrap(), offset);

        let result = unsafe {
            posix88_f::mman::munmap(
                address,
                map_size)
        };
        if result == -1 {
            let c_error = CString::new("Couldn't unmap file").unwrap();
            unsafe {
                stdio::perror(c_error.as_ptr());
            }
        }

        let diff: i64 = std::num::cast(map_size).unwrap();
        remaining_file_size -= diff;
        offset += diff;
    }

    unsafe {
        posix88_f::unistd::close(fd);
    }

    Ok(())
}

fn main() {
    let mut args = env::args();
    let mut stderr = std::io::stderr();
    if args.len() != 2 {
        writeln!(&mut stderr, "Usage: rexdump <file>").unwrap();
        process::exit(1);
    }
    let s : String = args.nth(1).unwrap();
    read_print_file(&s).unwrap();
}
