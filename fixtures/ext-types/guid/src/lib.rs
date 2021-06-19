// A trivial guid.
pub struct Guid(pub String);

fn get_guid(guid: Option<Guid>) -> Guid {
    match guid {
        Some(guid) => guid,
        None => Guid("NewGuid".to_string()),
    }
}

pub struct GuidHelper {
    pub guid: Guid,
    pub guids: Vec<Guid>,
}

fn get_guid_helper(vals: Option<GuidHelper>) -> GuidHelper {
    match vals {
        None => GuidHelper {
            guid: Guid("first-guid".to_string()),
            guids: vec![
                Guid("second-guid".to_string()),
                Guid("third-guid".to_string()),
            ],
        },
        Some(vals) => vals,
    }
}

// And we need a ViaFfi for them. Both of these boil down to "to and from a
// string" - this is the best markh could come up with, Maybe it should be
// a proc macro? Whatever, this serves the purpose for now.
use anyhow::Result;
use bytes::buf::{Buf, BufMut};
use std::convert::TryFrom;
use uniffi::{check_remaining, RustBuffer, ViaFfi};

#[macro_export]
macro_rules! viaffi_simple_string {
    ( $t:ident, $sel:ident, $to_string:expr, $from_string:expr ) => {
        unsafe impl ViaFfi for $t {
            type FfiType = RustBuffer;
            fn lower($sel) -> <$t as ViaFfi>::FfiType {
                RustBuffer::from_vec($to_string.into_bytes())
            }
            fn try_lift(v: Self::FfiType) -> Result<Self> {
                let v = v.destroy_into_vec();
                let s = unsafe { String::from_utf8_unchecked(v) };
                Ok($from_string(s)) // XXX - wire up errors
            }
            fn write<B: BufMut>(&$sel, buf: &mut B) {
                let s = &$to_string;
                let len = i32::try_from(s.len()).unwrap();
                buf.put_i32(len); // We limit strings to u32::MAX bytes
                buf.put(s.as_bytes());
            }
            fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
                check_remaining(buf, 4)?;
                let len = usize::try_from(buf.get_i32())?;
                check_remaining(buf, len)?;
                let bytes = &buf.chunk()[..len];
                let s = String::from_utf8(bytes.to_vec())?;
                buf.advance(len);
                Ok($from_string(s)) // XXX - wire up errors
            }
        }
    }
}

viaffi_simple_string!(Guid, self, self.0, |s| Self(s));

include!(concat!(env!("OUT_DIR"), "/guid.uniffi.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
