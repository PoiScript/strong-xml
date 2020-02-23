use jetscii::{bytes, BytesConst};
use lazy_static::lazy_static;
use std::borrow::Cow;

pub fn xml_escape<'a>(raw: &'a str) -> Cow<'a, str> {
    lazy_static! {
        static ref ESCAPE_BYTES: BytesConst = bytes!(b'<', b'>', b'&', b'\'', b'"');
    }

    let bytes = raw.as_bytes();

    if let Some(off) = ESCAPE_BYTES.find(bytes) {
        let mut result = String::with_capacity(raw.len());

        result.push_str(&raw[0..off]);

        let mut pos = off + 1;

        match bytes[pos - 1] {
            b'<' => result.push_str("&lt;"),
            b'>' => result.push_str("&gt;"),
            b'&' => result.push_str("&amp;"),
            b'\'' => result.push_str("&apos;"),
            b'"' => result.push_str("&quot;"),
            _ => unreachable!(),
        }

        while let Some(off) = ESCAPE_BYTES.find(&bytes[pos..]) {
            result.push_str(&raw[pos..pos + off]);

            pos += off + 1;

            match bytes[pos - 1] {
                b'<' => result.push_str("&lt;"),
                b'>' => result.push_str("&gt;"),
                b'&' => result.push_str("&amp;"),
                b'\'' => result.push_str("&apos;"),
                b'"' => result.push_str("&quot;"),
                _ => unreachable!(),
            }
        }

        result.push_str(&raw[pos..]);

        Cow::Owned(result)
    } else {
        Cow::Borrowed(raw)
    }
}

#[test]
fn test_escape() {
    assert_eq!(xml_escape("< < <"), "&lt; &lt; &lt;");
    assert_eq!(
        xml_escape("<script>alert('Hello XSS')</script>"),
        "&lt;script&gt;alert(&apos;Hello XSS&apos;)&lt;/script&gt;"
    );
}
