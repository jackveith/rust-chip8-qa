//chip emulator
use std::collections;
use std::io;
use std::fs::File;
use std::io::Read;
use std::fs::read_to_string;




fn main() {

    let mut memory:  [u8; 4096] = [0; 4096];
    let mut display: [[u8; 64]; 32] = [[0; 64]; 32];
    let mut pc: usize = 512;
    let mut ir: u16 = 0;
    let mut stack: Vec<u16> = Vec::new();
    let mut general_reg: [u8; 16] = [0; 16];



    read_program_into_memory(&mut memory); 
    load_font_into_memory(&mut memory);
    

    loop {




        let current_instruction = [memory[pc], memory[pc+1]];
        pc += 2;

        let current_instruction_nibbles = [
            current_instruction[0] >> 4,
            current_instruction[0] & 0xF,
            current_instruction[1] >> 4,
            current_instruction[1] & 0xF,
        ];

        //decode



        for a in current_instruction_nibbles {
            print!["{}, ", a];
        }

        println![];
        match current_instruction_nibbles {
            [  0,    0,  0xE,    0] => clear_screen(&mut display),
            [  1,    a,    b,    c] => jump(&mut pc, &mut stack, a, b, c),
            [  6, regx,    b,    c] => set_register(&mut general_reg, regx, b, c),
            [  7, regx,    b,    c] => add_register(&mut general_reg, regx, b, c),
            [0xA,    a,    b,    c] => set_index_register(&mut ir, a, b, c),
            [0xD, regx, regy,    n] => draw_sprite(&mut display, ir, &mut general_reg, &mut memory, regx, regy, n),
            [  8, regx, regy,   op] => register_operation(&mut general_reg, regx, regy, op),






            [0,0,0,0] => break,
                [_, _, _, _] => println!["failure to parse"],





        }
        print_display(&display);
    }

    //fetch
   
    //execute
}

fn register_operation (general_reg: &mut [u8], regx: u8, regy: u8, op: u8) {
    let x = regx as usize;
    let y = regy as usize;
    match op {
        0x0 => { general_reg[x] = general_reg[y] },
        0x1 => { general_reg[x] = general_reg[x] & general_reg[y] },
        0x2 => { general_reg[x] = general_reg[x] | general_reg[y] },
        0x3 => { general_reg[x] = general_reg[x] ^ general_reg[y] },
        0x4 => {
            let result: u16 = general_reg[x] as u16 + general_reg[y] as u16;
            general_reg[x] = result as u8;
            if result > 255 { general_reg[15] = 1 } else { general_reg[15] = 0 }
            
        }
        0x5 => {
            let result: u16 = general_reg[x] as u16 - general_reg[y] as u16;
            general_reg[x] = result as u8;
            if general_reg[x] > general_reg[y] { general_reg[15] = 1 } else { general_reg[15] = 0 }
            
        }
        0x6 => {
            general_reg[15] = general_reg[y] & 0x1;
            general_reg[x] = general_reg[y] >> 1;
        }
        0x7 => {
            let result: u16 = general_reg[y] as u16 - general_reg[x] as u16;
            general_reg[x] = result as u8;
            if general_reg[x] > general_reg[y] { general_reg[15] = 1 } else { general_reg[15] = 0 }
            
        }
        0xE => {
            general_reg[15] = general_reg[y] & 0x80;
            general_reg[x] = general_reg[y] << 1;
        }

        _ => {}
    }
}

fn draw_sprite (display: &mut [[u8; 64]; 32], ir: u16, general_reg: &mut [u8], memory: &mut [u8; 4096],  regx: u8, regy: u8, n: u8) {

    let x = general_reg[regx as usize];
    let y = general_reg[regy as usize];

    for row in 0..n {
        if row + y >= 32 { continue } 

        let mut sprite_row_pixels: [u8; 8] = [0; 8];
        let sprite_row_byte = memory[(ir + row as u16) as usize];

        for col in 0..8 {
            sprite_row_pixels[col] = sprite_row_byte >> (7 - col) & 0x1;  
        }

        for col in 0..8 {
            //skip if OOB
            if col + x >= 64 { continue }
            //skip if transparent
            if sprite_row_pixels[col as usize] == 0 { continue }
    
            
            let display_bit = display[(row + y) as usize][(col + x) as usize];
            if display_bit == 1 { 
                general_reg[15] = 1;
                display[(row + y) as usize][(col + x) as usize] = 0;
            }
            else { display[(row + y) as usize][(col + x) as usize] = 1; }

        }
    } 
}

fn set_index_register(ir: &mut u16, a: u8, b: u8, c: u8) {

    let reg_value: u16 = (a as u16) << 8 | (b as u16) << 4 | c as u16; 
    *ir = reg_value;

}

fn add_register(general_reg: &mut [u8], x: u8, b: u8, c: u8) {
    general_reg[x as usize] += b << 4 | c;
}

fn set_register(general_reg: &mut [u8], x: u8, b: u8, c: u8) {
    general_reg[x as usize] = b << 4 | c;
}


fn jump (pc: &mut usize, stack: &mut Vec<u16>,  a: u8, b: u8, c: u8) {
    stack.push(*pc as u16);
    let jump_location: u16 = (a as u16) << 8 | (b as u16) << 4 | c as u16; 
    *pc =  jump_location as usize;

}

fn clear_screen (display: &mut [[u8; 64]; 32]) {
    for i in 0..32 {
        for j in 0..64 {
            display[i][j] = 0;
        }
    } 
}

fn print_display(display: &[[u8; 64]; 32]) {

    for row in 0..32 {
        let mut printout = String::new();
        for i in 0..64 {
            if display[row][i] == 1 { printout.push('@')}
            else                    { printout.push('.')}
        }
        println!["{}", printout];

    }
} 


fn load_font_into_memory (mem: &mut [u8]) {
    
    let mut index = 80;
    const BUF_SIZE: usize = 20;

    let mut f = File::open("font.txt").unwrap();
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

    let mut f = File::open("IBM Logo.ch8").unwrap();
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    while index < 4096 {
        let read_count = f.read(&mut buffer).unwrap();
        for i in 0..read_count {
            if i < 8 { println!("{}", buffer[i]) };
            mem[index] = buffer[i];
            index += 1;
        }

        if read_count <  BUF_SIZE { break; }
        
    } 

}
