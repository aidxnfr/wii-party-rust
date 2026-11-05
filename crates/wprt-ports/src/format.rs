// port of engine/src/core/format.cpp

use std::ffi::CString;
use std::os::raw::c_char;

pub struct Cpu {
    pub r: [u32; 32],
    pub f: [f64; 32],
}

impl Default for Cpu {
    fn default() -> Self {
        Self { r: [0; 32], f: [0.0; 32] }
    }
}

pub trait Memory {
    fn rd8(&self, address: u32) -> u8;
    fn rd32(&self, address: u32) -> u32;
    fn rdf64(&self, address: u32) -> f64;
}

use libc::snprintf;

const K_FIRST_ARGUMENT_REGISTER: u32 = 3;
const K_LAST_ARGUMENT_REGISTER: u32 = 10;
const K_STACK_ARGUMENT_OFFSET: u32 = 8;
const K_FIRST_FLOAT_REGISTER: u32 = 1;
const K_LAST_FLOAT_REGISTER: u32 = 8;
const K_MAX_TEXT: usize = 4096;

// port of engine/src/core/format.cpp read_string
fn read_string(memory: &dyn Memory, address: u32) -> String {
    let mut text = Vec::new();
    let mut current = address;
    loop {
        let ch = memory.rd8(current);
        if ch == 0 || text.len() >= K_MAX_TEXT {
            break;
        }
        text.push(ch);
        current = current.wrapping_add(1);
    }
    String::from_utf8_lossy(&text).into_owned()
}

struct Arguments<'a> {
    c: &'a Cpu,
    memory: &'a dyn Memory,
    next_register: u32,
    next_float: u32,
    stack: u32,
}

impl<'a> Arguments<'a> {
    // port of engine/src/core/format.cpp Arguments::Arguments
    fn new(c: &'a Cpu, memory: &'a dyn Memory, first_register: u32) -> Self {
        Self {
            c,
            memory,
            next_register: first_register,
            next_float: K_FIRST_FLOAT_REGISTER,
            stack: c.r[1].wrapping_add(K_STACK_ARGUMENT_OFFSET),
        }
    }

    // port of engine/src/core/format.cpp Arguments::integer
    fn integer(&mut self) -> u32 {
        if self.next_register <= K_LAST_ARGUMENT_REGISTER {
            let value = self.c.r[self.next_register as usize];
            self.next_register += 1;
            return value;
        }
        let value = self.memory.rd32(self.stack);
        self.stack = self.stack.wrapping_add(4);
        value
    }

    // port of engine/src/core/format.cpp Arguments::wide
    fn wide(&mut self) -> u64 {
        if self.next_register <= K_LAST_ARGUMENT_REGISTER {
            if (self.next_register - K_FIRST_ARGUMENT_REGISTER) % 2 != 0 {
                self.next_register += 1;
            }
        } else if self.stack % 8 != 0 {
            self.stack = self.stack.wrapping_add(4);
        }
        let high = self.integer() as u64;
        let low = self.integer() as u64;
        (high << 32) | low
    }

    // port of engine/src/core/format.cpp Arguments::floating
    fn floating(&mut self) -> f64 {
        if self.next_float <= K_LAST_FLOAT_REGISTER {
            let value = self.c.f[self.next_float as usize];
            self.next_float += 1;
            return value;
        }
        if self.stack % 8 != 0 {
            self.stack = self.stack.wrapping_add(4);
        }
        let value = self.memory.rdf64(self.stack);
        self.stack = self.stack.wrapping_add(8);
        value
    }
}

// port of engine/src/core/format.cpp format_guest
pub fn format_guest(c: &Cpu, memory: &dyn Memory, format: u32, first_argument_register: u32) -> String {
    let mut arguments = Arguments::new(c, memory, first_argument_register);
    let text = read_string(memory, format).into_bytes();
    let mut output = Vec::new();
    let mut i = 0usize;
    while i < text.len() {
        if text[i] != b'%' {
            output.push(text[i]);
            i += 1;
            continue;
        }
        let mut spec = vec![b'%'];
        let mut length = 0i32;
        loop {
            i += 1;
            if i >= text.len() || !b"-+ #0123456789.*hlLzjt".contains(&text[i]) {
                break;
            }
            match text[i] {
                b'*' => spec.extend_from_slice(arguments.integer().to_string().as_bytes()),
                b'l' => length += 1,
                b'h' | b'z' | b'j' | b't' | b'L' => {}
                byte => spec.push(byte),
            }
        }
        if i >= text.len() {
            break;
        }
        let conversion = text[i];
        let mut buffer = [0u8; 512];
        match conversion {
            b'%' => output.push(b'%'),
            b'd' | b'i' => {
                let mut local = spec.clone();
                if length >= 2 {
                    local.extend_from_slice(b"lld");
                    let format = CString::new(local).unwrap();
                    let value = arguments.wide() as i64;
                    unsafe {
                        snprintf(buffer.as_mut_ptr() as *mut c_char, buffer.len(), format.as_ptr(), value);
                    }
                } else {
                    local.push(b'd');
                    let format = CString::new(local).unwrap();
                    let value = arguments.integer() as i32;
                    unsafe {
                        snprintf(buffer.as_mut_ptr() as *mut c_char, buffer.len(), format.as_ptr(), value);
                    }
                }
                if let Some(end) = buffer.iter().position(|&byte| byte == 0) {
                    output.extend_from_slice(&buffer[..end]);
                }
            }
            b'u' | b'x' | b'X' | b'o' => {
                let mut local = spec.clone();
                if length >= 2 {
                    local.extend_from_slice(b"ll");
                    local.push(conversion);
                    let format = CString::new(local).unwrap();
                    let value = arguments.wide();
                    unsafe {
                        snprintf(buffer.as_mut_ptr() as *mut c_char, buffer.len(), format.as_ptr(), value);
                    }
                } else {
                    local.push(conversion);
                    let format = CString::new(local).unwrap();
                    let value = arguments.integer();
                    unsafe {
                        snprintf(buffer.as_mut_ptr() as *mut c_char, buffer.len(), format.as_ptr(), value);
                    }
                }
                if let Some(end) = buffer.iter().position(|&byte| byte == 0) {
                    output.extend_from_slice(&buffer[..end]);
                }
            }
            b'c' => {
                let mut local = spec.clone();
                local.push(b'c');
                let format = CString::new(local).unwrap();
                let value = (arguments.integer() & 0xFF) as i32;
                unsafe {
                    snprintf(buffer.as_mut_ptr() as *mut c_char, buffer.len(), format.as_ptr(), value);
                }
                if let Some(end) = buffer.iter().position(|&byte| byte == 0) {
                    output.extend_from_slice(&buffer[..end]);
                }
            }
            b's' => {
                let address = arguments.integer();
                let mut local = spec.clone();
                local.push(b's');
                let format = CString::new(local).unwrap();
                let string = if address == 0 {
                    CString::new("(null)").unwrap()
                } else {
                    CString::new(read_string(memory, address)).unwrap()
                };
                unsafe {
                    snprintf(buffer.as_mut_ptr() as *mut c_char, buffer.len(), format.as_ptr(), string.as_ptr());
                }
                if let Some(end) = buffer.iter().position(|&byte| byte == 0) {
                    output.extend_from_slice(&buffer[..end]);
                }
            }
            b'p' => {
                let address = arguments.integer();
                let format = CString::new("0x%08x").unwrap();
                unsafe {
                    snprintf(buffer.as_mut_ptr() as *mut c_char, buffer.len(), format.as_ptr(), address);
                }
                if let Some(end) = buffer.iter().position(|&byte| byte == 0) {
                    output.extend_from_slice(&buffer[..end]);
                }
            }
            b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => {
                let mut local = spec.clone();
                local.push(conversion);
                let format = CString::new(local).unwrap();
                let value = arguments.floating();
                unsafe {
                    snprintf(buffer.as_mut_ptr() as *mut c_char, buffer.len(), format.as_ptr(), value);
                }
                if let Some(end) = buffer.iter().position(|&byte| byte == 0) {
                    output.extend_from_slice(&buffer[..end]);
                }
            }
            _ => {
                output.extend_from_slice(&spec);
                output.push(conversion);
            }
        }
        i += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}
