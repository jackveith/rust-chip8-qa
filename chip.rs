//chip emulator
use std::collections;
use std::io;
use std::fs::File;
use std::io::Read;





fn main() {

    #[allow(dead_code)]
    let mut memory:  [u8; 4096] = [0; 4096];
    let mut _display: [u32; 64] = [0; 64];
    let mut _pc: i32 = 0;
    let mut _ir: u16 = 0;
    #[allow(unused_variables)]
    let mut stack: Vec<u16> = Vec::new();
    let mut regs: [u8; 16] = [0; 16];



    //load program from file into memory
    println!("{}","lol");
    read_program_into_memory(&mut memory); 
    
    for i in 500..560 {
        println!("{}", memory[i]);
    }


    //fetch
    
    


    //decode




    //execute
}

fn read_program_into_memory(mem: &mut [u8]) {

    let mut f = File::open("lol.txt").unwrap();
    let mut index = 512;
    let mut buffer: [u8; 32] = [0; 32];

    while index < 4096 {
        let read_count = f.read(&mut buffer).unwrap();
        println!("{}", read_count);
        for i in 0..read_count {
            mem[index] = buffer[i];
            index += 1;
        } 
        
    } 

}
