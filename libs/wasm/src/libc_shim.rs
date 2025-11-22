use std::alloc::{GlobalAlloc, Layout, System};
use std::ffi::{c_void, c_int, c_char};
use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let align = std::mem::align_of::<usize>();
    let layout = Layout::from_size_align(size + align, align).unwrap();
    let ptr = System.alloc(layout);
    if ptr.is_null() {
        return ptr as *mut c_void;
    }
    *(ptr as *mut usize) = size;
    ptr.add(align) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() { return; }
    let align = std::mem::align_of::<usize>();
    let real_ptr = (ptr as *mut u8).sub(align);
    let size = *(real_ptr as *mut usize);
    let layout = Layout::from_size_align(size + align, align).unwrap();
    System.dealloc(real_ptr, layout);
}

#[no_mangle]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    let total_size = nmemb * size;
    let ptr = malloc(total_size);
    if !ptr.is_null() {
        ptr::write_bytes(ptr, 0, total_size);
    }
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return malloc(size);
    }
    if size == 0 {
        free(ptr);
        return ptr::null_mut();
    }
    
    let align = std::mem::align_of::<usize>();
    let real_ptr = (ptr as *mut u8).sub(align);
    let old_size = *(real_ptr as *mut usize);
    
    let new_ptr = malloc(size);
    if !new_ptr.is_null() {
        let copy_size = std::cmp::min(old_size, size);
        ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
        free(ptr);
    }
    new_ptr
}

#[no_mangle]
pub extern "C" fn abort() {
    panic!("abort called");
}

// String functions
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    ptr::copy(src as *const u8, dest as *mut u8, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void {
    ptr::write_bytes(s as *mut u8, c as u8, n);
    s
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int {
    let s1 = s1 as *const u8;
    let s2 = s2 as *const u8;
    for i in 0..n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return (a as c_int) - (b as c_int);
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 { break; }
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char {
    let mut i = 0;
    while i < n {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            // Pad with zeros
            for j in (i+1)..n {
                *dest.add(j) = 0;
            }
            break;
        }
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    let mut i = 0;
    loop {
        let a = *s1.add(i) as u8;
        let b = *s2.add(i) as u8;
        if a != b {
            return (a as c_int) - (b as c_int);
        }
        if a == 0 { break; }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int {
    for i in 0..n {
        let a = *s1.add(i) as u8;
        let b = *s2.add(i) as u8;
        if a != b {
            return (a as c_int) - (b as c_int);
        }
        if a == 0 { break; }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strdup(s: *const c_char) -> *mut c_char {
    let len = strlen(s);
    let ptr = malloc(len + 1) as *mut c_char;
    if !ptr.is_null() {
        strcpy(ptr, s);
    }
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn memchr(s: *const c_void, c: c_int, n: usize) -> *mut c_void {
    let s = s as *const u8;
    let c = c as u8;
    for i in 0..n {
        if *s.add(i) == c {
            return s.add(i) as *mut c_void;
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
    let mut last = ptr::null_mut();
    let mut i = 0;
    loop {
        let ch = *s.add(i);
        if ch == c as c_char {
            last = s.add(i) as *mut c_char;
        }
        if ch == 0 { break; }
        i += 1;
    }
    last
}

#[no_mangle]
pub unsafe extern "C" fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    let len = strlen(dest);
    strcpy(dest.add(len), src);
    dest
}

#[no_mangle]
pub extern "C" fn fputc(_c: c_int, _stream: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn fputs(_s: *const c_char, _stream: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn fwrite(
    _ptr: *const c_void,
    _size: usize,
    nmemb: usize,
    _stream: *mut c_void,
) -> usize {
    // Pretend that all items were written successfully.
    nmemb
}
#[no_mangle]
pub extern "C" fn fclose(_stream: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn fdopen(_fd: c_int, _mode: *const c_char) -> *mut c_void { ptr::null_mut() }

#[no_mangle]
pub extern "C" fn isspace(c: c_int) -> c_int {
    if c == 32 || (c >= 9 && c <= 13) { 1 } else { 0 }
}
#[no_mangle]
pub extern "C" fn isdigit(c: c_int) -> c_int {
    if c >= 48 && c <= 57 { 1 } else { 0 }
}
#[no_mangle]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    if (c >= 65 && c <= 90) || (c >= 97 && c <= 122) { 1 } else { 0 }
}
#[no_mangle]
pub extern "C" fn isalnum(c: c_int) -> c_int {
    if isalpha(c) != 0 || isdigit(c) != 0 { 1 } else { 0 }
}
#[no_mangle]
pub extern "C" fn isprint(c: c_int) -> c_int {
    if c >= 32 && c < 127 { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn close(_fd: c_int) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn read(_fd: c_int, _buf: *mut c_void, _count: usize) -> isize { 0 }
#[no_mangle]
pub extern "C" fn write(_fd: c_int, _buf: *const c_void, _count: usize) -> isize { 0 }
#[no_mangle]
pub extern "C" fn dup(_oldfd: c_int) -> c_int { -1 }

#[no_mangle]
pub extern "C" fn clock() -> isize { 0 }
#[no_mangle]
pub extern "C" fn time(_tloc: *mut isize) -> isize { 0 }

#[no_mangle]
pub extern "C" fn iswspace(wc: c_int) -> c_int { isspace(wc) }
#[no_mangle]
pub extern "C" fn iswalpha(wc: c_int) -> c_int { isalpha(wc) }
#[no_mangle]
pub extern "C" fn iswalnum(wc: c_int) -> c_int { isalnum(wc) }
#[no_mangle]
pub extern "C" fn iswdigit(wc: c_int) -> c_int { isdigit(wc) }
#[no_mangle]
pub extern "C" fn towlower(wc: c_int) -> c_int { 
    if wc >= 65 && wc <= 90 { wc + 32 } else { wc }
}
#[no_mangle]
pub extern "C" fn towupper(wc: c_int) -> c_int {
    if wc >= 97 && wc <= 122 { wc - 32 } else { wc }
}
