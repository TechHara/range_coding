use std::io::{Read, Result, Write};
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub mod read;
pub mod table;

pub const BUFFER_SIZE: usize = 1 << 15;
pub const INIT_RANGE: u64 = 1 << 32;
pub const EMIT_MASK: u64 = 0xFF << 24;
pub const SHIFT_MASK: u64 = 0xFFFFFF;
pub const EMIT_SHIFT: u64 = 24;
pub const READJUST_THRESHOLD: u64 = 1 << 16;
pub const N_READJUST_EMIT: u64 = 2;

pub fn write_freq<T: Copy>(freq: &[T], mut writer: impl Write) -> Result<usize> {
    let buf = unsafe { from_raw_parts(freq.as_ptr() as *const u8, std::mem::size_of_val(freq)) };
    writer.write_all(buf)?;
    Ok(buf.len())
}

pub fn read_freq<T: Copy>(mut reader: impl Read, freq: &mut [T]) -> Result<usize> {
    let buf = unsafe { from_raw_parts_mut(freq.as_ptr() as *mut u8, std::mem::size_of_val(freq)) };
    reader.read_exact(buf)?;
    Ok(buf.len())
}

pub fn shift(mut x: u64) -> u64 {
    x &= SHIFT_MASK;
    x << 8
}

pub fn update_range(
    mut low: u64,
    mut range: u64,
    start: u64,
    size: u64,
    total: u64,
    verbosity: u32,
) -> Result<(u64, u64)> {
    debug_assert!(size != 0);
    debug_assert!(range != 0);
    range /= total;
    low += start * range;
    range *= size;

    if verbosity > 0 {
        eprintln!(
            "low: 0x{:08x}\thigh: 0x{:08x}\trange: 0x{:08x}",
            low,
            low + range,
            range
        );
    }

    Ok((low, range))
}
