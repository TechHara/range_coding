use crate::{read_freq, write_freq};
use std::io::{Read, Result, Write};

pub struct Table {
    table: Vec<(u16, u16)>, // (start, size)
    total: u64,
}

impl Table {
    pub fn new(xs: &[u8]) -> Self {
        debug_assert!(xs.len() <= u16::MAX as usize);
        let mut freq = [0; 256];
        for x in xs {
            freq[*x as usize] += 1;
        }
        Self::from_freq(&freq)
    }

    pub fn from_freq(freq: &[u16; 256]) -> Self {
        let mut table: Vec<(u16, u16)> = freq.iter().map(|f| (0, *f)).collect();
        let mut s = 0u16;
        for (start, size) in &mut table {
            *start = s;
            s += *size;
        }

        Self {
            table,
            total: s as u64,
        }
    }

    pub fn read(read: impl Read) -> Result<Self> {
        let mut freq = [0u16; 256];
        read_freq(read, &mut freq)?;
        Ok(Self::from_freq(&freq))
    }

    pub fn write(&self, write: impl Write) -> Result<usize> {
        let freq: Vec<u16> = self.table.iter().map(|(_, size)| *size).collect();
        write_freq(&freq, write)
    }

    pub fn start(&self, x: u8) -> u64 {
        self.table[x as usize].0 as u64
    }

    pub fn size(&self, x: u8) -> u64 {
        self.table[x as usize].1 as u64
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn search(&self, value: u16) -> u8 {
        let idx = self.table.partition_point(|(s, _)| *s <= value);
        (idx - 1) as u8
    }

    pub fn create_lookup(&self) -> Vec<u8> {
        let mut lookup = Vec::with_capacity(self.total as usize);
        for (x, (_, f)) in self.table.iter().enumerate() {
            let n = lookup.len();
            lookup.resize(n + *f as usize, x as u8);
        }
        debug_assert!(lookup.len() == self.total as usize);
        lookup
    }

    pub fn trivial(&self) -> Option<u8> {
        self.table
            .iter()
            .position(|(_, f)| *f as u64 == self.total())
            .map(|idx| idx as u8)
    }
}
