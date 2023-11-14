use range_coding::read::ReadAtMost;
use range_coding::table::Table;
use range_coding::{
    shift, update_range, BUFFER_SIZE, EMIT_MASK, EMIT_SHIFT, INIT_RANGE, N_READJUST_EMIT,
    READJUST_THRESHOLD,
};
use std::io::{stdin, stdout, Cursor, Result, Write};

fn usage(program: &str) {
    eprintln!("Encode data from stdin and output to stdout");
    eprintln!("Usage: {program} [VERBOSITY]");
    eprintln!("\tVERBOSITY is non-negative integer");
    std::process::exit(-1);
}

struct Encoder<W> {
    write: W,
    low: u64,
    range: u64,
    verbosity: u32,
}

impl<W: Write> Encoder<W> {
    fn new(write: W, verbosity: u32) -> Self {
        Self {
            write,
            low: 0,
            range: INIT_RANGE,
            verbosity,
        }
    }

    fn encode(&mut self, start: u64, size: u64, total: u64) -> Result<()> {
        (self.low, self.range) =
            update_range(self.low, self.range, start, size, total, self.verbosity)?;

        while self.low & EMIT_MASK == (self.low + self.range) & EMIT_MASK {
            self.emit_byte()?;
        }

        if self.range < READJUST_THRESHOLD {
            if self.verbosity > 0 {
                eprintln!("range readjusting");
            }
            for _ in 0..N_READJUST_EMIT {
                self.emit_byte()?;
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

    fn emit_byte(&mut self) -> Result<()> {
        debug_assert!(self.range != 0);
        let byte = (self.low & EMIT_MASK) >> EMIT_SHIFT;
        if self.verbosity > 0 {
            eprintln!("emit: {}", byte);
        }
        self.write.write_all(&[byte as u8])?;
        self.low = shift(self.low);
        self.range <<= 8;

        if self.verbosity > 0 {
            eprintln!(
                "low: 0x{:08x}\thigh: 0x{:08x}\trange: 0x{:08x}",
                self.low,
                self.low + self.range,
                self.range
            );
        }
        Ok(())
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

    let mut buf = vec![0; BUFFER_SIZE];
    let mut reader = stdin();
    let mut writer = stdout();

    loop {
        let mut cursor = Cursor::new(Vec::new());
        let n = reader.read_at_most(&mut buf)?;
        if n == 0 {
            break;
        }
        let buf = &buf[..n];
        let table = Table::new(buf);
        if let Some(x) = table.trivial() {
            if verbosity > 0 {
                eprintln!("trivial block with 0x{:02} x {}", x, n);
            }
            writer.write_all(&(0 as u32).to_le_bytes())?;
            writer.write_all(&(n as u16).to_le_bytes())?;
            writer.write_all(&[x])?;
        } else {
            let _n_freq = table.write(&mut cursor)?;
            let mut encoder = Encoder::new(&mut cursor, verbosity);

            for (_idx, x) in buf.iter().enumerate() {
                let start = table.start(*x);
                let size = table.size(*x);
                if verbosity > 0 {
                    eprintln!("x: {}\tstart: {}\tsize: {}", *x, start, size);
                }
                encoder.encode(start, size, n as u64)?;
            }

            while encoder.low != 0 {
                encoder.emit_byte()?;
            }

            // write to stdout
            cursor.flush()?;
            let mut out_buf = cursor.into_inner();
            let out_size = out_buf.len() + 4;
            out_buf.resize(out_size, 0);
            writer.write_all(&(out_size as u32).to_le_bytes())?;
            writer.write_all(&out_buf)?;
        }

        if n < BUFFER_SIZE {
            break;
        }
    }

    Ok(())
}
