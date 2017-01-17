/// This implements our own line codec.
/// Instead of operating solely in bytes,
/// like the tokio-core toy example,
/// we use this codec to
/// decode bytes to strings (splitting at newline boundaries).

use std::io::Write;
use std::str;
pub use tokio_core::io::{Codec, EasyBuf};

pub struct LineCodec;

impl Codec for LineCodec {
    type In = String;
    type Out = String;

    fn decode(&mut self, buf: &mut EasyBuf) -> ::std::io::Result<Option<Self::In>> {
        Ok(buf.as_ref().iter() // Iterate as a byte slice
           .position(|&b| b == b'\n') // Find a newline
           .map(|pos| buf.drain_to(pos+1)) // Split the line off of the buffer, including \n
           .and_then(|line| {
               str::from_utf8(&line.as_ref()[..line.len()-1]).ok().map(|s| s.to_owned())
           })) // Convert to String, IGNORING invalid utf-8 lines.
    }
    fn encode(&mut self, msg: String, buf: &mut Vec<u8>) -> ::std::io::Result<()> {
        buf.write(msg.as_bytes()).and_then(|_| buf.write(b"\n")).map(|_| ())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::From;

    macro_rules! buf {
        ($str:expr) => {
            EasyBuf::from($str.bytes().collect::<Vec<u8>>())
        }
    }

    #[test]
    fn test_linecodec_decode() {
        let mut l = LineCodec;
        let mut buf = buf!("hello\nworld\n");
        assert_eq!(l.decode(&mut buf).unwrap(), Some("hello".to_owned()));
        assert_eq!(l.decode(&mut buf).unwrap(), Some("world".to_owned()));
        assert_eq!(l.decode(&mut buf).unwrap(), None);

        assert_eq!(l.decode(&mut buf!("hell")).unwrap(), None);
        assert_eq!(l.decode(&mut buf!("")).unwrap(), None);
    }

    #[test]
    fn test_linecodec_encode() {
        let mut l = LineCodec;
        let mut buf = vec![];
        l.encode("hello".to_owned(), &mut buf).unwrap();
        assert_eq!(&buf, b"hello\n");
        l.encode("world".to_owned(), &mut buf).unwrap();
        assert_eq!(&buf, b"hello\nworld\n");
    }
}
