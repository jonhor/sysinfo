use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

const CPU_COUNT_FILE: &str = "/sys/devices/system/cpu/present";
const CPU_PROC_FILE: &str = "/proc/stat";

pub struct CpuInfo {
    file: File,
    buf: String,
    cpu_count: u32,
}

pub struct CpuStats;

fn get_cpu_count() -> u32 {
    let s = fs::read_to_string(CPU_COUNT_FILE).unwrap();
    let l = s.lines().next().unwrap();

    let (left, right) = l.split_once('-').unwrap();
    let start: u32 = left.parse().unwrap();
    let end: u32 = right.parse().unwrap();

    // sysfs cpu uses zero based indexing
    (end - start) + 1
}

impl CpuInfo {
    pub fn new() -> io::Result<Self> {
        Ok(CpuInfo {
            file: File::open(CPU_PROC_FILE)?,
            buf: String::new(),
            cpu_count: get_cpu_count(),
        })
    }

    fn stats() {}
}
