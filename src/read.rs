use std::io::{Read, Result};

pub trait ReadAtMost {
    /// read as many bytes as it can to fill up the buffer
    /// will not throw error if EOF encountered
    /// will throw for any other error
    fn read_at_most(&mut self, buf: &mut [u8]) -> Result<usize>;
}

impl<R: Read> ReadAtMost for R {
    fn read_at_most(&mut self, mut buf: &mut [u8]) -> Result<usize> {
        let mut total = 0;
        while !buf.is_empty() {
            let n = self.read(buf)?;
            if n == 0 {
                // must be EOF
                break;
            }
            total += n;
            buf = &mut buf[n..];
        }
        Ok(total)
    }
}
