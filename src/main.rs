use std::io;
use std::io::prelude::*;
use std::fs::File;


fn main() -> io::Result<()> {
    let mut memory: [u8; 4096]=[0; 4096];
    let mut f = File::open("tests/exp")?;
    f.read(&mut memory)?;
    println!("{}", memory[4]);
    Ok(())
}
