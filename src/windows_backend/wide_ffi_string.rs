
use std::ffi::{OsStr, OsString};
use std::os::windows::prelude::{OsStrExt, OsStringExt};

use smallvec::SmallVec;
use windows::core::{IntoParam, Abi, Param, PWSTR, PCWSTR};

// This type is only in the Windows backend for a couple reasons:
// - Even though macOS uses UTF-16 like Windows does, it has a CFString create function that takes a
//   UTF-8 string. So I don't think it is needed on macOS.
// - The standard library's OsStr::encode_wide() function is only available on Windows.

pub(crate) struct WideFfiString<A: ::smallvec::Array<Item=u16>> {
    buffer: SmallVec<A>,
}

impl<A: ::smallvec::Array<Item=u16>> Clone for WideFfiString<A> {
    fn clone(&self) -> Self {
        Self { buffer: self.buffer.clone() }
    }

    fn clone_from(&mut self, source: &Self) {
        self.buffer.clone_from(&source.buffer)
    }
}

impl<A: ::smallvec::Array<Item=u16>> WideFfiString<A> {
    pub(crate) fn len(&self) -> usize {
        return self.buffer.len() - 1; // don't include null-terminator
    }

    pub(crate) fn as_slice(&self) -> &[u16] {
        &self.buffer[0..self.buffer.len() - 1]
    }

    pub(crate) fn as_ptr(&self) -> *const u16 {
        self.buffer.as_ptr()
    }
}

impl<A: ::smallvec::Array<Item=u16>> WideFfiString<A> {
    pub(crate) fn new(s: &str) -> Self {
        let mut new_self = Self { buffer: SmallVec::<A>::new() };
        new_self.reset(s);
        new_self
    }

    pub(crate) fn reset(&mut self, s: &str) {
        self.buffer.clear();
        self.buffer.extend(
            OsStr::new(s).encode_wide().map(|c| if c == 0 { b'?' as u16 } else { c })
        );
        self.buffer.push(0);
    }
}

impl<'a, A: ::smallvec::Array<Item=u16>> IntoParam<'a, PWSTR> for &WideFfiString<A> {
    fn into_param(self) -> Param<'a, PWSTR> {
        Param::Owned(PWSTR(self.buffer.as_ptr() as *mut _))
    }
}

impl<'a, A: ::smallvec::Array<Item=u16>> IntoParam<'a, PCWSTR> for &WideFfiString<A> {
    fn into_param(self) -> Param<'a, PCWSTR> {
        Param::Owned(PCWSTR(self.buffer.as_ptr() as *mut _))
    }
}
