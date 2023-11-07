use std::fmt::{Display, Formatter};

use libc::{c_char, geteuid, readlink, PATH_MAX};

pub(crate) fn base_name(name: &str) -> &str {
    match name.rsplit_once('/') {
        Some((_, base_name)) => base_name,
        None => name,
    }
}

pub(crate) fn is_root() -> bool {
    effective_user_id() == 0
}

fn effective_user_id() -> u32 {
    unsafe { geteuid() }
}

pub(crate) fn pid_link(pid: i32, base_name: &str) -> String {
    let link = format!("/proc/{pid}/{base_name}");

    read_link(&link)
}

fn read_link(link: &str) -> String {
    let link_str = NullTerminatedString::from(link);
    let mut output = NullTerminatedString::new(PATH_MAX as usize);

    loop {
        let read_amount = unsafe { readlink(link_str.as_ptr(), output.as_mut_ptr(), output.len()) };
        if read_amount < 0 {
            break;
        }

        if (read_amount as usize) < output.len() {
            output.set_read(read_amount as usize);
            break;
        }

        output.grow();
    }

    output.unwrap()
}

struct NullTerminatedString(String, usize);

impl NullTerminatedString {
    fn new(size: usize) -> Self {
        let str: String = "\0".repeat(size + 1);

        Self(str, 0)
    }

    fn set_read(&mut self, amount: usize) {
        self.1 = amount;
    }

    fn grow(&mut self) {
        let extension = NullTerminatedString::new(self.len()).unwrap();

        self.0.pop().unwrap();
        self.0.push_str(&extension);
        self.0.push('\0');
    }

    fn len(&self) -> usize {
        self.0.len() - 1
    }

    unsafe fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast::<c_char>()
    }

    unsafe fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr().cast::<c_char>()
    }

    fn unwrap(mut self) -> String {
        self.0.truncate(self.1);

        self.0
    }
}

impl From<String> for NullTerminatedString {
    fn from(value: String) -> Self {
        let mut new = Self(value.clone(), 0);

        new.0.push('\0');

        new
    }
}

impl From<&str> for NullTerminatedString {
    fn from(value: &str) -> Self {
        let mut new = Self(value.to_owned(), 0);

        new.0.push('\0');

        new
    }
}

impl Display for NullTerminatedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
