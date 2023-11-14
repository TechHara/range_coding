use std::io::{stdin, stdout, Cursor, Error, ErrorKind, Read, Result, Write};

use range_coding::{
    read::ReadAtMost, shift, table::Table, update_range, BUFFER_SIZE, EMIT_MASK, INIT_RANGE,
    N_READJUST_EMIT, READJUST_THRESHOLD,
};

fn usage(program: &str) {
    eprintln!("Decode from stdin and output to stdout");
    eprintln!("Usage: {program} [VERBOSITY]");
    eprintln!("\tVERBOSITY is non-negative integer");

    std::process::exit(-1);
}

struct Decoder<I> {
    iter: I,
    code: u64,
    low: u64,
    range: u64,
    verbosity: u32,
}

impl<I: Iterator<Item = Result<u8>>> Decoder<I> {
    fn new(iter: I, verbosity: u32) -> Self {
        Self {
            iter,
            code: 0,
            low: 0,
            range: 1,
            verbosity,
        }
    }

    fn init(&mut self) -> Result<()> {
        for _ in 0..4 {
            self.append_byte()?;
        }
        Ok(())
    }

    fn decode(&mut self, start: u64, size: u64, total: u64) -> Result<()> {
        (self.low, self.range) =
            update_range(self.low, self.range, start, size, total, self.verbosity)?;

        while self.low & EMIT_MASK == (self.low + self.range) & EMIT_MASK {
            self.append_byte()?;
        }

        if self.range < READJUST_THRESHOLD {
            if self.verbosity > 0 {
                eprintln!("range readjusting");
            }
            for _ in 0..N_READJUST_EMIT {
                self.append_byte()?;
            }
            // reduce range to cap at INIT_RANGE
            self.range = INIT_RANGE - self.low;
            if self.verbosity > 0 {
                eprintln!(
                    "low: 0x{:08x}\thigh: 0x{:08x}\trange: 0x{:08x}",
                    self.low,
                    self.low + self.range,
                    self.range
                );
            }
        }
        Ok(())
    }

    fn append_byte(&mut self) -> Result<()> {
        let byte = self.iter.next().unwrap()?;
        self.code = shift(self.code) | (byte as u64);
        self.low = shift(self.low);
        self.range <<= 8;
        if self.verbosity > 0 {
            eprintln!(
                "low: 0x{:08x}\thigh: 0x{:08x}\trange: 0x{:08x}",
                self.low,
                self.low + self.range,
                self.range
            );
            eprintln!("emit: {}", byte);
        }

        Ok(())
    }

    fn get_value(&self, total: u64) -> u64 {
        (self.code - self.low) / (self.range / total)
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    let mut verbosity = 0;

    if let Some(v) = args.next() {
        verbosity = v.parse().unwrap();
    }
    if args.next().is_some() {
        usage(&program);
    }

    let mut reader = stdin();
    let mut writer = stdout();

    loop {
        let mut buf = [0; 4];
        match reader.read_at_most(&mut buf)? {
            0 => {
                break;
            }
            4 if u32::from_le_bytes(buf) == 0 => {
                reader.read_exact(&mut buf[..3])?;
                let n = u16::from_le_bytes(buf[..2].try_into().unwrap()) as usize;
                let x = buf[2];
                if verbosity > 0 {
                    eprintln!("trivial block with 0x{:02} x {}", x, n);
                }
                writer.write_all(&vec![x; n])?;
                if n < BUFFER_SIZE {
                    break;
                } else {
                    continue;
                }
            }
            4 => {}
            _ => {
                return Err(Error::new(ErrorKind::UnexpectedEof, ""));
            }
        }
        let block_size = u32::from_le_bytes(buf) as usize;
        let mut buf = vec![0; block_size];
        reader.read_exact(&mut buf)?;

        let mut cursor = Cursor::new(buf);
        let table = Table::read(&mut cursor)?;
        let lookup = table.create_lookup();
        let mut decoder = Decoder::new(cursor.bytes(), verbosity);
        decoder.init()?;

        let mut start;
        let mut size;
        let total = table.total();
        let mut n = 0;
        let mut out_cursor = Cursor::new(Vec::<u8>::new());

        loop {
            let v = decoder.get_value(total);
            let idx = lookup[v as usize];
            debug_assert!(idx == table.search(v as u16));
            start = table.start(idx);
            size = table.size(idx);
            if verbosity > 0 {
                eprintln!("x: {}\tstart: {}\tsize: {}", idx, start, size);
            }
            out_cursor.write_all(&[idx])?;
            n += 1;
            if n >= total {
                break;
            }
            decoder.decode(start, size, total)?;
        }

        let out_buf = out_cursor.into_inner();
        writer.write_all(&out_buf)?;

        if (total as usize) < BUFFER_SIZE {
            break;
        }
    }

    Ok(())
}
