
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
#[cfg(windows)]
use std::os::windows::prelude::{OsStrExt, OsStringExt};

use smallvec::SmallVec;
#[cfg(windows)]
use windows::core::{IntoParam, Abi, Param, PWSTR, PCWSTR};

#[derive(Clone)]
pub(crate) struct WideFfiString<A: ::smallvec::Array<Item=u16>> {
    buffer: SmallVec<A>,
}

impl<A: ::smallvec::Array<Item=u16>> WideFfiString<A> {
    pub(crate) fn new(s: &str) -> Self {
        let mut buffer = SmallVec::<A>::new();
        buffer.extend(OsStr::new(s).encode_wide().map(|c| if c == 0 { b'?' as u16 } else { c }));
        buffer.push(0);
        Self { buffer }
    }
}

impl<A: ::smallvec::Array<Item=u16>> Debug for WideFfiString<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WideFfiString")
            .field("buffer", &OsString::from_wide(&self.buffer))
            .finish()
    }
}

#[cfg(windows)]
impl<'a, A: ::smallvec::Array<Item=u16>> IntoParam<'a, PWSTR> for &WideFfiString<A> {
    fn into_param(self) -> Param<'a, PWSTR> {
        Param::Owned(PWSTR(self.buffer.as_ptr() as *mut _))
    }
}

#[cfg(windows)]
impl<'a, A: ::smallvec::Array<Item=u16>> IntoParam<'a, PCWSTR> for &WideFfiString<A> {
    fn into_param(self) -> Param<'a, PCWSTR> {
        Param::Owned(PCWSTR(self.buffer.as_ptr() as *mut _))
    }
}
