//chip emulator
use std::collections;
use std::io;
use std::fs::File;
use std::io::Read;
use std::fs::read_to_string;




fn main() {

    #[allow(dead_code)]
    let mut memory:  [u8; 4096] = [0; 4096];
    let mut _display: [[u8; 64]; 32] = [[0; 64]; 32];
    let mut pc: usize= 512;
    let mut _ir: u16 = 0;
    #[allow(unused_variables)]
    let mut stack: Vec<u16> = Vec::new();
    let mut general_reg: [u8; 16] = [0; 16];



    //load program from file into memory
    read_program_into_memory(&mut memory); 
    //load font file into memory
    load_font_into_memory(&mut memory);
    

    //fetch
   
    let current_instruction = (memory[pc], memory[pc+1]);
    pc += 2;
    

    


    //decode

    



    //execute
}

fn print_display() {} 


fn load_font_into_memory (mem: &mut [u8]) {
    
    let mut index = 80;
    const BUF_SIZE: usize = 20;

    let mut f = File::open("out.txt").unwrap();
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    while index < 240 {
        let read_count = f.read(&mut buffer).unwrap();
        for i in 0..read_count {
            mem[index] = buffer[i];
            index += 1;
        }

        if read_count <  BUF_SIZE { break; }
        
    } 

    
     
}

fn read_program_into_memory(mem: &mut [u8]) {

    let mut index = 512;
    const BUF_SIZE: usize = 64;

    let mut f = File::open("lol.txt").unwrap();
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    while index < 4096 {
        let read_count = f.read(&mut buffer).unwrap();
        for i in 0..read_count {
            mem[index] = buffer[i];
            index += 1;
        }

        if read_count <  BUF_SIZE { break; }
        
    } 

}
