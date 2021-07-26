use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, SeekFrom};
use std::num;

use thiserror::Error;

/// tries to emulate behaviour of free(1) without having to call an external process everytime you want to check memory usage

const MEM_PROC_FILE: &str = "/proc/meminfo";

// same buf size as free(1). /proc/meminfo shouldn't be larger than 2048 (linux 5.4) but rust's string type will automatically increase capacity in case this limit is ever reached
const INITIAL_BUF_SIZE: usize = 2048;

const INDEX_KEYS: [&str; 5] = ["MemTotal", "MemFree", "Buffers", "Cached", "SReclaimable"];

#[derive(Error, Debug)]
pub enum MemErr {
    #[error("encountered unexpected format: {0}")]
    UnexpectedFormat(&'static str),
    #[error("key {0} is missing from {}", MEM_PROC_FILE)]
    MissingValue(String),
    #[error(transparent)]
    FileHandlingError(#[from] io::Error),
    #[error(transparent)]
    ParseMemoryValue(#[from] num::ParseIntError),
}

type MemResult<T> = Result<T, MemErr>;

pub struct MemInfo {
    file: File,
    buf: String,
    stats: MemStats,
    lookup: BTreeMap<usize, String>,
    cache: HashMap<String, usize>,
}

#[derive(Debug, Default)]
pub struct MemStats {
    total: usize,
    free: usize,
    used: usize,
}

fn build_lookup_table(file: &mut File, buf: &mut String) -> MemResult<BTreeMap<usize, String>> {
    file.read_to_string(buf)?;
    file.seek(SeekFrom::Start(0))?;

    let mut keys = Vec::new();
    for line in buf.lines() {
        let key = line
            .split(':')
            .nth(0)
            .ok_or(MemErr::UnexpectedFormat("missing colon (:)"))?;

        keys.push(key);
    }

    let mut lookup = BTreeMap::new();
    for key in &INDEX_KEYS {
        if !keys.contains(key) {
            return Err(MemErr::MissingValue(String::from(*key)));
        }

        // panic:
        // every key should have a position in keys because
        // this is the path where contains returned true
        let index = keys.iter().position(|s: &&str| *s == *key).unwrap();
        lookup.insert(index, String::from(*key));
    }

    buf.clear();

    Ok(lookup)
}

impl MemInfo {
    pub fn new() -> MemResult<Self> {
        let mut file = File::open(MEM_PROC_FILE)?;
        let mut buf = String::with_capacity(INITIAL_BUF_SIZE);

        let lookup = build_lookup_table(&mut file, &mut buf)?;

        let mut cache = HashMap::with_capacity(lookup.len());

        for value_name in lookup.values() {
            cache.insert(value_name.clone(), 0);
        }

        Ok(MemInfo {
            file,
            buf,
            stats: MemStats::default(),
            lookup,
            cache,
        })
    }

    fn parse_kb_value_from_line(&self, s: &str) -> MemResult<usize> {
        let end = s.len() - 3;
        let start = s
            .get(..end)
            .ok_or(MemErr::UnexpectedFormat("line is too short"))?
            .rfind(char::is_whitespace)
            .ok_or(MemErr::UnexpectedFormat(
                "line doesn't contain any whitespace",
            ))?
            + 1;

        Ok(s[start..end].parse::<usize>()?)
    }

    pub fn stats(&mut self) -> MemResult<&MemStats> {
        self.file.read_to_string(&mut self.buf)?;
        // set file position to 0 to read from start again next time stats is called
        self.file.seek(SeekFrom::Start(0))?;

        let mut lines = self.buf.lines();
        let mut last_pos = 0;

        // update cache with new values
        for (n_line, value_name) in &self.lookup {
            let advance = n_line - last_pos;
            let n = if advance == 0 { 0 } else { advance - 1 };

            // panic:
            // the only way this panics is if the number of lines in /proc/meminfo
            // changed between constructing MemInfo and calling this function,
            // which should never happen
            let line = lines.nth(n).unwrap();
            let value = self.parse_kb_value_from_line(line)?;

            // panic:
            // new() made sure that all value names are already keys in this map
            *self.cache.get_mut(value_name).unwrap() = value;
            last_pos = *n_line;
        }

        // update stats from cache
        self.stats.total = self.cache["MemTotal"];
        self.stats.free = self.cache["MemFree"];
        self.stats.used = self.cache["MemTotal"]
            - self.cache["MemFree"]
            - self.cache["Buffers"]
            - self.cache["Cached"]
            - self.cache["SReclaimable"];

        // clear keeps capacity but sets len to 0 to avoid appends next time stats is called
        self.buf.clear();

        Ok(&self.stats)
    }
}
