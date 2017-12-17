use std::fs::File;
use std::io::prelude::*;
use std::env;

fn literal_or_register(addr: u16, registers: &[u16]) -> u16 {
    if addr >= 32768 {
        let x = registers[addr as usize - 32768];
        // println!("Register {0} contains (0x{1:04x} / {1})", addr, x);
        x
    } else {
        addr
    }
}

fn register(addr: u16, registers: &mut [u16]) -> &mut u16 {
    let reg = addr - 32768;
    &mut registers[reg as usize]
}

fn dissasemble(word: u16) -> String {
    match word {
        // halt: 0
        // stop execution and terminate the program
        0 => "halt".into(),
        // set: 1 a b
        // set register <a> to the value of <b>
        1 => "set".into(),
        // push: 2 a
        // push <a> onto the stack
        2 => "push".into(),
        // pop: 3 a
        // remove the top element from the stack and write it into <a>; empty stack = error
        3 => "pop".into(),
        // eq: 4 a b c
        // set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
        4 => "eq".into(),
        // gt: 5 a b c
        5 => "gt".into(),
        // set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
        // jmp: 6 a
        // jump to <a>
        6 => "jmp".into(),
        // jt: 7 a b
        // if <a> is nonzero, jump to <b>
        7 => "jump-true".into(),
        // jf: 8 a b
        // if <a> is zero, jump to <b>
        8 => "jump-false".into(),
        // add: 9 a b c
        // assign into <a> the sum of <b> and <c> (modulo 32768)
        9 => "add".into(),
        // mult: 10 a b c
        // store into <a> the product of <b> and <c> (modulo 32768)
        10 => "mult".into(),
        // mod: 11 a b c
        // store into <a> the remainder of <b> divided by <c>
        11 => "mod".into(),
        // and: 12 a b c
        // stores into <a> the bitwise and of <b> and <c>
        12 => "and".into(),
        // or: 13 a b c
        // stores into <a> the bitwise or of <b> and <c>
        13 => "or".into(),
        // not: 14 a b
        // stores 15-bit bitwise inverse of <b> in <a>
        14 => "not".into(),
        // rmem: 15 a b
        // read memory at address <b> and write it to <a>
        15 => "rmem".into(),
        // wmem: 16 a b
        // write the value from <b> into memory at address <a>
        16 => "wmem".into(),
        // call: 17 a
        // write the address of the next instruction to the stack and jump to <a>
        17 => "call".into(),
        // ret: 18
        // remove the top element from the stack and jump to it; empty stack = halt
        18 => "ret".into(),
        // out: 19 a
        // write the character represented by ascii code <a> to the terminal
        19 => "out".into(),
        // in: 20 a
        // read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard and trust that they will be fully read
        // todo
        // noop: 21
        // no operation
        21 => "nop".into(),
        32768 => "Register 0".into(),
        32769 => "Register 1".into(),
        32770 => "Register 2".into(),
        32771 => "Register 3".into(),
        32772 => "Register 4".into(),
        32773 => "Register 5".into(),
        32774 => "Register 6".into(),
        32775 => "Register 7".into(),
        x if x < 128 => format!("printable {} ({})", x, (x as u8 as char).escape_default()),
        x => format!("literal {}", x),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Need bin argument");
    }
    let filename = &args[1];
    println!("Loading {}", filename);
    let mut file = File::open(filename).expect("Can't open file");
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data).expect("Can't read");
    println!("Read {} bytes", data.len());
    let mut words: Vec<u16> = data.chunks(2)
        .map(|x| x[0] as u16 + (x[1] as u16 * 256))
        .collect();
    println!("Read {} words", words.len());
    for (i, v) in words.iter().enumerate() {
        println!("0x{0:04x}: 0x{1:04x} (0d{1}) ; {2}", i, v, dissasemble(*v));
    }
    let mut registers = vec![0u16; 8];
    let mut stack: Vec<u16> = Vec::new();
    let mut pc = 0;
    loop {
        let op = words[pc];
        // println!("Executing {} ({}) at 0x{:04x}", op, dissasemble(op), pc);
        match op {
            // halt: 0
            // stop execution and terminate the program
            0 => return,
            // set: 1 a b
            // set register <a> to the value of <b>
            1 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = b;
                pc = pc + 3;
            }
            // push: 2 a
            // push <a> onto the stack
            2 => {
                let a = literal_or_register(words[pc + 1], &registers);
                stack.push(a);
                pc = pc + 2;
            }
            // pop: 3 a
            // remove the top element from the stack and write it into <a>; empty stack = error
            3 => {
                let a = register(words[pc + 1], &mut registers);
                *a = stack.pop().expect("Empty stack");
                pc = pc + 2;
            }
            // eq: 4 a b c
            // set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
            4 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let c = literal_or_register(words[pc + 3], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = if b == c { 1 } else { 0 };
                pc = pc + 4;
            }
            // gt: 5 a b c
            // set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
            5 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let c = literal_or_register(words[pc + 3], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = if b > c { 1 } else { 0 };
                pc = pc + 4;
            }
            // jmp: 6 a
            // jump to <a>
            6 => {
                pc = literal_or_register(words[pc + 1], &registers) as usize;
            }
            // jt: 7 a b
            // if <a> is nonzero, jump to <b>
            7 => {
                let a = literal_or_register(words[pc + 1], &registers);
                let b = literal_or_register(words[pc + 2], &registers);
                if a != 0 {
                    pc = b as usize;
                } else {
                    pc = pc + 3;
                }
            }
            // jf: 8 a b
            // if <a> is zero, jump to <b>
            8 => {
                let a = literal_or_register(words[pc + 1], &registers);
                let b = literal_or_register(words[pc + 2], &registers);
                if a == 0 {
                    pc = b as usize;
                } else {
                    pc = pc + 3;
                }
            }
            // add: 9 a b c
            // assign into <a> the sum of <b> and <c> (modulo 32768)
            9 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let c = literal_or_register(words[pc + 3], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = b.wrapping_add(c) & 0x7FFF;
                pc = pc + 4;
            }
            // mult: 10 a b c
            // store into <a> the product of <b> and <c> (modulo 32768)
            10 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let c = literal_or_register(words[pc + 3], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = b.wrapping_mul(c) & 0x7FFF;
                pc = pc + 4;
            }
            // mod: 11 a b c
            // store into <a> the remainder of <b> divided by <c>
            11 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let c = literal_or_register(words[pc + 3], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = (b % c) & 0x7FFF;
                pc = pc + 4;
            }
            // and: 12 a b c
            // stores into <a> the bitwise and of <b> and <c>
            12 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let c = literal_or_register(words[pc + 3], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = (b & c) & 0x7FFF;
                pc = pc + 4;
            }
            // or: 13 a b c
            // stores into <a> the bitwise or of <b> and <c>
            13 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let c = literal_or_register(words[pc + 3], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = (b | c) & 0x7FFF;
                pc = pc + 4;
            }
            // not: 14 a b
            // stores 15-bit bitwise inverse of <b> in <a>
            14 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = (!b) & 0x7FFF;
                pc = pc + 3;
            }
            // rmem: 15 a b
            // read memory at address <b> and write it to <a>
            15 => {
                let b = literal_or_register(words[pc + 2], &registers);
                let a = register(words[pc + 1], &mut registers);
                *a = words[b as usize];
                pc = pc + 3;
            }
            // wmem: 16 a b
            // write the value from <b> into memory at address <a>
            16 => {
                let a = literal_or_register(words[pc + 1], &registers);
                let b = literal_or_register(words[pc + 2], &registers);
                words[a as usize] = b;
                pc = pc + 3;
            }
            // call: 17 a
            // write the address of the next instruction to the stack and jump to <a>
            17 => {
                stack.push((pc + 2) as u16);
                pc = literal_or_register(words[pc + 1], &registers) as usize;
            }
            // ret: 18
            // remove the top element from the stack and jump to it; empty stack = halt
            18 => {
                pc = stack.pop().unwrap() as usize;
            }
            // out: 19 a
            // write the character represented by ascii code <a> to the terminal
            19 => {
                let a = literal_or_register(words[pc + 1], &registers);
                //println!("'{}'", ch as u8 as char);
                print!("{}", a as u8 as char);
                pc = pc + 2;
            }
            // in: 20 a
            // read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard and trust that they will be fully read
            20 => {
                let a = register(words[pc + 1], &mut registers);
                println!("Reading...");
                *a = std::io::stdin().bytes().next().unwrap().unwrap() as u16;
                println!("Read {}", *a);
                pc = pc + 2;
            }
            // noop: 21
            // no operation
            21 => {
                pc = pc + 1;
            }
            x => panic!("Unhandled opcode {}", x),
        }
    }
}
