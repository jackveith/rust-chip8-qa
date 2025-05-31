//chip8 interpreter
use std::fs::File;
use std::io::Read;
use rand::prelude::random;
use std::time::Duration;
use std::{ thread, time };
use std::collections::HashSet;

use sdl2;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::{ Keycode, Scancode };


const DISPLAY_WIDTH:  usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const PIXEL_SIZE: usize = 10;
const MEMORY_SIZE: usize = 4096;
const PROGRAM_OFFSET: usize = 512;
const FONT_OFFSET: usize = 0x50;
const FONT_SIZE: usize = 80;
const TIMER_CLOCK_FREQUENCY: Duration = Duration::from_micros(16666);
const PROGRAM_TITLE: &str = "programs/snake.ch8";
const FRAMERATE: usize = 60;

const SCANCODES: [Scancode; 16] = [
    Scancode::Num1, Scancode::Num2, Scancode::Num3, Scancode::Num4,
    Scancode::Q,    Scancode::W,    Scancode::E,    Scancode::R,
    Scancode::A,    Scancode::S,    Scancode::D,    Scancode::F,
    Scancode::Z,    Scancode::X,    Scancode::C,    Scancode::V,
];


fn main() {

    //chip8 system 
    let mut memory:  [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let mut display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT] = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    let mut pc: usize = PROGRAM_OFFSET; //program counter
    let mut ir: u16 = 0; //index register
    let mut stack: Vec<u16> = Vec::new();
    let mut g_register: [u8; 16] = [0; 16];
    let mut timer_delay: u8 = 0;
    let mut timer_sound: u8 = 0;

    let mut timer_last_update: time::Instant = time::Instant::now();
    let mut frame_start: time::Instant = time::Instant::now();
    //let mut keystate: [u8; 16] = [0; 16];

    //SDL2 video and input system initialization
    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let window = sdl_video_subsystem.window("chip8", (DISPLAY_WIDTH * PIXEL_SIZE) as u32, (DISPLAY_HEIGHT * PIXEL_SIZE) as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas()
        .build()
        .unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();


    read_program_into_memory(&mut memory); 
    load_font_into_memory(&mut memory);

    'mainloop: loop {


        //checks if the program has been exited or if escape has been
        //pressed in order to exit cleanly
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'mainloop },
                _ => {}
            }
        }



        //FETCH
        let current_instruction = [memory[pc], memory[pc+1]];
        pc += 2;

        let current_instruction_nibbles = [
            current_instruction[0] >>   4,
            current_instruction[0] &  0xF,
            current_instruction[1] >>   4,
            current_instruction[1] &  0xF,
        ];

        
        frame_start = time::Instant::now();


        //DECODE AND EXECUTE MATCH
        match current_instruction_nibbles {
            [    0,    0,  0xE,    0] => clear_screen(&mut display),
            [    0,    0,  0xE,  0xE] => return_from_subroutine(&mut pc, &mut stack),
            [    1,    a,    b,    c] => jump_to_address(&mut pc, a, b, c),
            [    2,    a,    b,    c] => enter_into_subroutine(&mut pc, &mut stack, a, b, c),
            [    3, regx,    b,    c] => skip_instruction_eq_literal(&mut pc, &g_register, regx, b, c),
            [    4, regx,    b,    c] => skip_instruction_ne_literal(&mut pc, &g_register, regx, b, c),
            [    5, regx, regy,    0] => skip_instruction_eq_register(&mut pc, &g_register, regx, regy),
            [    6, regx,    b,    c] => set_register(&mut g_register, regx, b, c),
            [    7, regx,    b,    c] => add_register(&mut g_register, regx, b, c),
            [    8, regx, regy,   op] => register_operation(&mut g_register, regx, regy, op),
            [    9, regx, regy,    0] => skip_instruction_ne_register(&mut pc, &g_register, regx, regy),
            [  0xA,    a,    b,    c] => set_index_register(&mut ir, a, b, c),
            [  0xB,    a,    b,    c] => jump_v0_offset(&mut pc, &g_register, a, b, c),
            [  0xC, regx,    b,    c] => set_register_random(&mut g_register, regx, b, c),
            [  0xD, regx, regy,    n] => draw_sprite(&mut display, ir, &mut g_register, &mut memory, regx, regy, n),
            [  0xE, regx,    9,  0xE] => skip_instruction_key_pressed(&mut pc, &event_pump, &g_register, regx),
            [  0xE, regx,  0xA,    1] => skip_instruction_key_not_pressed(&mut pc, &event_pump, &g_register, regx),
            [  0xF, regx,    b,    c] => special_register_operation(&mut g_register, &mut memory, &mut timer_delay, &mut timer_sound, &mut ir, regx, b, c),


            [0,0,0,0] => (),
                [_, _, _, _] => {
                    println!["failure to parse"];
                    for nb in current_instruction_nibbles {
                        print!("{:?}, ", nb);
                    } println!();
                    break 'mainloop;
                }
            ,

        }

        //console display
        print_display(&mut display);

        //SDL2 display
        pressed_scancode_set(&event_pump);
        update_canvas(&mut canvas, &display);
        canvas.present();
        //thread::sleep(time::Duration::from_millis(300));


        //Update both timers on a 60Hz clock
        if time::Instant::now() > timer_last_update + TIMER_CLOCK_FREQUENCY  {
            timer_last_update = time::Instant::now();
            decrement_timers(&mut timer_sound, &mut timer_delay);

        }

        //thread::sleep(time::Duration::from_millis(16));
        

    } //END MAINLOOP

    //
    //
    //
    //
}

fn update_canvas(canvas: &mut Canvas<Window>, display: &[[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT]) -> Result<(), String>   {

    canvas.set_draw_color(Color::RGB(0, 0, 0)); 
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 255, 255)); 
    for row in 0..DISPLAY_HEIGHT {
        for col in 0..DISPLAY_WIDTH {
            if display[row][col] == 1 {
            
                canvas.fill_rect(sdl2::rect::Rect::new((col * PIXEL_SIZE) as i32,
                                                       (row * PIXEL_SIZE) as i32,
                                                       PIXEL_SIZE as u32,
                                                       PIXEL_SIZE as u32))?; 
            } 
        }
    }
    Ok(())
}



fn pressed_scancode_set(event_pump: &sdl2::EventPump) -> HashSet<Scancode> {
    
    let c: HashSet<Scancode> = event_pump.keyboard_state().pressed_scancodes().collect();
    for s in c {
        print!("{:?}, ", s);
    }

    event_pump.keyboard_state().pressed_scancodes().collect()

}

fn decrement_timers(timer_sound: &mut u8, timer_delay: &mut u8) {
    if *timer_sound > 0 { *timer_sound -= 1 }
    if *timer_delay > 0 { *timer_delay -= 1 }
}

//SKIPPING ON KEY EVENTS

fn skip_instruction_key_pressed(pc: &mut usize, event_pump: &sdl2::EventPump, g_register: &[u8], regx: u8) {
    println!("{:?}", &SCANCODES[g_register[regx as usize] as usize]);
    if pressed_scancode_set(event_pump).contains(&SCANCODES[g_register[regx as usize] as usize]) { *pc += 2 }
}

fn skip_instruction_key_not_pressed(pc: &mut usize, event_pump: &sdl2::EventPump, g_register: &[u8], regx: u8) {
    println!("{:?}", &SCANCODES[g_register[regx as usize] as usize]);
    if !pressed_scancode_set(event_pump).contains(&SCANCODES[g_register[regx as usize] as usize]) { *pc += 2 }
}




//JUMPING AND SUBROUTINES

fn jump_to_address (pc: &mut usize, a: u8, b: u8, c: u8) {
    let jump_location: u16 = (a as u16) << 8 | (b as u16) << 4 | c as u16; 
    *pc =  jump_location as usize;
}

fn jump_v0_offset (pc: &mut usize, g_register: &[u8], a: u8, b: u8, c: u8) {
    let jump_location: u16 = ((a as u16) << 8 | (b as u16) << 4 | c as u16) + g_register[0] as u16; 
    *pc =  jump_location as usize;
}

fn enter_into_subroutine (pc: &mut usize, stack: &mut Vec<u16>, a: u8, b:u8, c: u8) {
    stack.push(*pc as u16);
    let jump_location: u16 = (a as u16) << 8 | (b as u16) << 4 | c as u16; 
    *pc =  jump_location as usize;
}

fn return_from_subroutine (pc: &mut usize, stack: &mut Vec<u16>) {
    let offset = stack.pop().unwrap();
    *pc = offset as usize;
}



//BASIC REGISTER OPERATIONS

fn set_index_register(ir: &mut u16, a: u8, b: u8, c: u8) {
    let reg_value: u16 = ((a as u16) << 8) | ((b as u16) << 4) | c as u16; 
    *ir = reg_value;
}

fn set_register(g_register: &mut [u8], x: u8, b: u8, c: u8) {
    g_register[x as usize] = (b << 4) | c;
}

fn set_register_random (g_register: &mut [u8; 16], regx: u8, b: u8, c: u8) {
    let rnum: u8 = random::<u8>();
    g_register[regx as usize] = rnum & ((b << 4) + c);
}

fn add_register(g_register: &mut [u8], x: u8, b: u8, c: u8) {
    let sum: u16 = g_register[x as usize] as u16 + (b << 4 | c) as u16;
    g_register[x as usize] = sum as u8;
}





//CONDITIONAL INSTRUCTION SKIPPING
//3XNN
fn skip_instruction_eq_literal (pc: &mut usize, g_register: &[u8], regx: u8, b: u8, c: u8) {
    if g_register[regx as usize] == ((b << 4) + c) { *pc += 2} 
}

fn skip_instruction_ne_literal (pc: &mut usize, g_register: &[u8], regx: u8, b: u8, c: u8) {
    if g_register[regx as usize] != ((b << 4) + c) { *pc += 2} 
}

fn skip_instruction_eq_register (pc: &mut usize, g_register: &[u8], regx: u8, regy: u8) {
    if g_register[regx as usize] == g_register[regy as usize] { *pc += 2} 
}

fn skip_instruction_ne_register (pc: &mut usize, g_register: &[u8], regx: u8, regy: u8) {
    if g_register[regx as usize] != g_register[regy as usize] { *pc += 2} 
}


//OPERATIONS ON REGISTER VALUES

fn register_operation (g_register: &mut [u8], regx: u8, regy: u8, op: u8) {
    let x = regx as usize;
    let y = regy as usize;
    match op {
        //LD
        0x0 => { g_register[x] = g_register[y] },
        //AND (all are bitwise)
        0x1 => { g_register[x] = g_register[x] & g_register[y] },
        //OR
        0x2 => { g_register[x] = g_register[x] | g_register[y] },
        //XOR
        0x3 => { g_register[x] = g_register[x] ^ g_register[y] },
        //ADD
        0x4 => {
            let result: u16 = g_register[x] as u16 + g_register[y] as u16;
            g_register[x] = result as u8;
            if result > 255 { g_register[15] = 1 } else { g_register[15] = 0 }
            
        }
        //SUB
        0x5 => {
            if g_register[x] > g_register[y] { g_register[15] = 1 } else { g_register[15] = 0 }
            let result: i16 = g_register[x] as i16 - g_register[y] as i16;
            g_register[x] = result as u8;
            
        }
        //SHR
        0x6 => {
            g_register[15] = g_register[y] & 0x1;
            g_register[x] = g_register[y] >> 1;
        }
        //SUBN
        0x7 => {
            let result: i16 = g_register[y] as i16 - g_register[x] as i16;
            g_register[x] = result as u8;
            if g_register[x] > g_register[y] { g_register[15] = 1 } else { g_register[15] = 0 }
            
        }
        //SHL
        0xE => {
            g_register[15] = g_register[y] & 0x80;
            g_register[x] = g_register[y] << 1;
        }

        _ => ()
    }
}


//SPECIAL REGISTER OPERATIONS
fn special_register_operation (g_register: &mut [u8], memory: &mut [u8], timer_delay: &mut u8, timer_sound: &mut u8, ir: &mut u16, regx: u8, b: u8, c: u8) {
    let x = regx as usize;
    match (b, c) {

        //TIMER INTO REG
        (0,   7) => g_register[x] = *timer_delay,
        //TODO: KEYPRESS TO REG
        (0, 0xA) => (),
        //REG INTO DELAY
        (1,   5) => *timer_delay = g_register[x],
        //REG INTO SOUND
        (1,   8) => *timer_sound = g_register[x],
        //IR = IR + REG
        (1, 0xE) => *ir = *ir + g_register[x] as u16,
        //TODO: IMPLEMENT LOAD SPRITE ADDR TO IR
        (2,   9) => {
            let location = FONT_OFFSET + (g_register[x] * 20) as usize;
            *ir = location as u16;
        },
        //BINARY CODED DECIMAL
        (3,   3) => {
            let digits = [g_register[x] / 100, g_register[x] /10 % 10, g_register[x] % 10];
            for i in 0..3 { memory[*ir as usize + i as usize] = digits[i] }
        }
        //FILL REG INTO MEM
        (5,   5) => {
            for i in 0..(regx + 1) { memory[(*ir + i as u16) as usize] = g_register[i as usize] }
            *ir = *ir + 1 + regx as u16;   
        },
        //FILL MEM INTO REG
        (6,   5) => { 
            for i in 0..(regx + 1) { g_register[i as usize] = memory[(*ir + i as u16) as usize] }
            *ir = *ir + 1 + regx as u16;
        }

        (_,_) => ()
    }
}

fn draw_sprite (display: &mut [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT], ir: u16, g_register: &mut [u8], memory: &mut [u8; MEMORY_SIZE],  regx: u8, regy: u8, n: u8) {

    let x: u16 = g_register[regx as usize] as u16;
    let y: u16 = g_register[regy as usize] as u16;

    for row in 0..(n as u16) {
        if row + y >= DISPLAY_HEIGHT as u16 { continue } 

        let mut sprite_row_pixels: [u8; 8] = [0; 8];
        let sprite_row_byte = memory[(ir + row as u16) as usize];

        for col in 0..8 {
            sprite_row_pixels[col] = sprite_row_byte >> (7 - col) & 0x1;  
        }

        for col in 0..8 {
            //skip if OOB
            if col + x >= DISPLAY_WIDTH as u16 { continue }
            //skip if transparent
            if sprite_row_pixels[col as usize] == 0 { continue }
    
            
            let display_bit = display[(row + y) as usize][(col + x) as usize];
            if display_bit == 1 { 
                g_register[15] = 1;
                display[(row + y) as usize][(col + x) as usize] = 0;
            }
            else { display[(row + y) as usize][(col + x) as usize] = 1; }

        }
    } 
}



//DISPLAY FUNCTIONS

fn clear_screen (display: &mut [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT]) {
    for i in 0..DISPLAY_HEIGHT {
        for j in 0..DISPLAY_WIDTH {
            display[i][j] = 0;
        }
    } 
}

fn print_display(display: &[[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT]) {
    let mut printout = String::new();
    for row in 0..DISPLAY_HEIGHT {
        for i in 0..DISPLAY_WIDTH {
            if display[row][i] == 1 { printout.push('@')}
            else                    { printout.push(' ')}
        }
        printout.push('\n');
    }
    print!("{}", printout);
} 





//LOADING FILES INTO SIMULATED MEMORY
//
fn load_font_into_memory (mem: &mut [u8]) {
    
    let mut index = FONT_OFFSET;
    const BUF_SIZE: usize = 20;

    let mut f = File::open("font.txt").unwrap();
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    while index < FONT_OFFSET + FONT_SIZE {
        let read_count = f.read(&mut buffer).unwrap();
        for i in 0..read_count {
            mem[index] = buffer[i];
            index += 1;
        }

        if read_count <  BUF_SIZE { break; }
    } 
} 

fn read_program_into_memory(mem: &mut [u8]) {


    let mut index = PROGRAM_OFFSET;
    const BUF_SIZE: usize = 64;

    let mut f = File::open(PROGRAM_TITLE).unwrap();
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    while index < MEMORY_SIZE {
        let read_count = f.read(&mut buffer).unwrap();
        for i in 0..read_count {
            if i < 8 { println!("{}", buffer[i]) };
            mem[index] = buffer[i];
            index += 1;
        }

        if read_count <  BUF_SIZE { break; }
        
    } 

}
