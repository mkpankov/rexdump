#![feature(page_size)]

extern crate libc;
extern crate num;

#[macro_use(syscall)]
extern crate syscall;

use num::traits::NumCast;

use std::env;
use std::io::{self, Write};
use std::process;

fn errno() -> i32 {
    io::Error::last_os_error().raw_os_error().unwrap_or(-1)
}

fn print_error(error: &str) {
    let mut stderr = std::io::stderr();
    writeln!(&mut stderr, "{}", error).unwrap();
}

mod c_helpers {
    use libc::types::os::arch::c95 as c95_t;
    use libc::funcs::c95 as c95_f;
    use std::ffi::CStr;
    use std::str;

    pub fn strerror(errno: c95_t::c_int) -> &'static str {
        let s = unsafe {
            c95_f::string::strerror(errno)
        };
        unsafe {
            str::from_utf8(CStr::from_ptr(s).to_bytes()).unwrap()
        }
    }
}

mod fd {
    use libc::types::os::arch::c95 as c95_t;

    use libc::consts::os::posix88 as posix88_c;
    use libc::funcs::posix88 as posix88_f;
    use libc::types::os::arch::posix01;

    use std::ffi::CString;
    use std::mem;

    pub struct Fd {
        raw_fd: c95_t::c_int,
    }

    impl Fd {
        pub fn open(path: &str) -> Result<Fd, i32> {
            let c_path = CString::new(path).unwrap();
            let fd = unsafe {
                syscall!(
                    OPEN,
                    c_path.as_ptr(),
                    posix88_c::O_RDONLY,
                    0)
            } as i32;
            if fd > -1000 && fd < 0 {
                return Err(-fd);
            }

            Ok(Fd { raw_fd: fd })
        }
        pub fn raw(&self) -> c95_t::c_int {
            self.raw_fd
        }
        pub fn get_size(&self) -> Result<i64, i32> {
            let mut file_info: posix01::stat = unsafe {
                mem::uninitialized()
            };
            let result = unsafe {
                posix88_f::stat_::fstat(self.raw(), &mut file_info)
            };
            if result == -1 {
                return Err(::errno());
            }
            Ok(file_info.st_size)
        }
    }


    impl Drop for Fd {
        fn drop(&mut self) {
            unsafe {
                syscall!(
                    CLOSE,
                    self.raw_fd);
            }
        }
    }
}

mod memory_map {
    use std::slice;
    use num::traits::NumCast;
    use libc::consts::os::posix88 as posix88_c;
    use libc::types::common::c95::c_void;
    use libc::consts::os::extra;
    use libc::types::os::arch::c95 as c95_t;

    pub struct MemoryMap {
        address: *mut c_void,
        length: u64,
    }

    impl MemoryMap {
        pub fn map(fd: c95_t::c_int, offset: i64, length: u64)
                   -> Result<MemoryMap, i32>
        {
            let address = unsafe {
                syscall!(
                    MMAP,
                    0 as *mut c_void,
                    length,
                    posix88_c::PROT_READ,
                    posix88_c::MAP_PRIVATE
                  | extra::MAP_POPULATE,
                    fd,
                    offset)
            } as *mut c_void;
            let address_code = address as i32;
            if address_code > -1000 && address_code < 0 {
                return Err(-address_code);
            }

            Ok(MemoryMap { address: address, length: length })
        }
        pub fn as_bytes(&self) -> &[u8] {
            unsafe {
                slice::from_raw_parts(
                    self.address as *const u8, NumCast::from(self.length).unwrap())
            }
        }
    }

    impl Drop for MemoryMap {
        fn drop(&mut self) {
            unsafe {
                syscall!(
                    MUNMAP,
                    self.address,
                    self.length)
            };
        }
    }
}


fn print_offset(offset: i64) {
    print!("{:08x}  ", offset)
}

fn print_hex(buffer: &[u8], line_width: i64) {
    for (i, c) in (0..).zip(buffer.iter()) {
        print!("{:02x} ", c);
        if i == line_width / 2 - 1 {
            print!(" ");
        }
    }
}

fn align_delimiter(line_size_current: i64, line_size_full: i64) {
    for _ in line_size_current..line_size_full {
        print!("   ");
    }

    if line_size_current < line_size_full / 2 {
        print!(" ");
    }

    print!(" |");
}

fn print_chars(buffer: &[u8]) {
    for c in buffer {
        let is_print = unsafe {
            libc::funcs::c95::ctype::isprint(
                num::traits::NumCast::from(*c).unwrap()) != 0
        };
        if is_print {
            print!("{}", *c as char);
        } else {
            print!(".")
        }
    }
}

fn print_contents(buffer: &[u8], buffer_size: i64, offset: i64) {
    if buffer_size == 0 {
        return;
    };

    let line_width_elements = 16;
    let mut remaining_buffer_size = buffer_size;
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

        print_offset(current_offset);

        print_hex(
            line,
            line_width_elements);

        align_delimiter(line_size, line_width_elements);

        print_chars (line);

        println!("|");

        current_offset += line_size;
        remaining_buffer_size -= line_size;
    }

    print_offset(current_offset);

    println!("");
}

fn read_print_file(path: &str) -> Result<(), ()> {
    let maybe_fd = fd::Fd::open(path);
    let fd: fd::Fd;
    match maybe_fd {
        Ok(f) => fd = f,
        Err(errno) => {
            print_error(
                &format!("Couldn't open file: {}", c_helpers::strerror(errno)));
            return Err(());
        }
    }
    let mut remaining_file_size = fd.get_size().unwrap();
    let page_size : i64 = NumCast::from(std::env::page_size()).unwrap();
    let mut offset = 0;
    while remaining_file_size > 0 {
        let map_size: u64 = NumCast::from(
            if remaining_file_size > page_size {
                page_size
            } else {
                remaining_file_size
            }).unwrap();
        let maybe_memory_map = memory_map::MemoryMap::map(
            fd.raw(), offset, map_size);
        let memory_map;
        match maybe_memory_map {
            Ok(m) => memory_map = m,
            Err(errno) => {
                print_error(
                    &format!("Couldn't read file: {}", c_helpers::strerror(errno)));
                return Err(());
            }
        }

        let buffer = memory_map.as_bytes();

        print_contents(buffer, NumCast::from(map_size).unwrap(), offset);

        let diff: i64 = NumCast::from(map_size).unwrap();
        remaining_file_size -= diff;
        offset += diff;
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
