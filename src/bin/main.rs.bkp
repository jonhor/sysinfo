mod cpu;
mod mem;

use mem::MemInfo;

use std::{thread, time};

fn main() {
    cpu::CpuInfo::new();
}

fn get_mem_info() {
    let mut mem_info = MemInfo::new().unwrap();

    for _ in 0..10 {
        let stats = mem_info.stats();
        println!("{:?}", stats);

        thread::sleep(time::Duration::from_secs(5));
    }
}
