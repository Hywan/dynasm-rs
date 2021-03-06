#![feature(plugin)]
#![plugin(dynasm)]

#[macro_use]
extern crate dynasmrt;
use dynasmrt::DynasmApi;

extern crate itertools;
use itertools::Itertools;

use std::io::{Read, BufRead, Write, stdin, stdout, BufReader, BufWriter};
use std::env;
use std::fs::File;
use std::slice;
use std::mem;
use std::u8;

const TAPE_SIZE: usize = 30000;

#[cfg(target_arch = "x86_64")]
dynasm!(ops
    ; .alias a_state, rcx
    ; .alias a_current, rdx
    ; .alias a_begin, r8
    ; .alias a_end, r9
    ; .alias retval, rax
);

macro_rules! prologue {
    ($ops:ident) => {{
        let start = $ops.offset();
        dynasm!($ops
            ; sub rsp, 0x28
            ; mov [rsp + 0x30], rcx
            ; mov [rsp + 0x40], r8
            ; mov [rsp + 0x48], r9
        );
        start
    }};
}

macro_rules! epilogue {
    ($ops:ident, $e:expr) => {dynasm!($ops
        ; mov retval, $e
        ; add rsp, 0x28
        ; ret
    );};
}

macro_rules! call_extern {
    ($ops:ident, $addr:expr) => {dynasm!($ops
        ; mov [rsp + 0x38], rdx
        ; mov rax, QWORD $addr as _
        ; call rax
        ; mov rcx, [rsp + 0x30]
        ; mov rdx, [rsp + 0x38]
        ; mov r8,  [rsp + 0x40]
        ; mov r9,  [rsp + 0x48]
    );};
}

struct State<'a> {
    pub input: Box<BufRead + 'a>,
    pub output: Box<Write + 'a>,
    tape: [u8; TAPE_SIZE]
}

struct Program {
    code: dynasmrt::ExecutableBuffer,
    start: dynasmrt::AssemblyOffset,
}


impl Program {
    fn compile(program: &[u8]) -> Result<Program, &'static str> {
        let mut ops = dynasmrt::Assembler::new();
        let mut loops = Vec::new();
        let mut code = program.iter().cloned().multipeek();

        let start = prologue!(ops);

        while let Some(c) = code.next() {
            match c {
                b'<' => {
                    let amount = code.take_while_ref(|x| *x == b'<').count() + 1;
                    dynasm!(ops
                        ; sub a_current, (amount % TAPE_SIZE) as _
                        ; cmp a_current, a_begin
                        ; jae >wrap
                        ; add a_current, TAPE_SIZE as _
                        ;wrap:
                    );
                },
                b'>' => {
                    let amount = code.take_while_ref(|x| *x == b'>').count() + 1;
                    dynasm!(ops
                        ; add a_current, (amount % TAPE_SIZE) as _
                        ; cmp a_current, a_end
                        ; jb >wrap
                        ; sub a_current, TAPE_SIZE as _
                        ;wrap:
                    );
                },
                b'+' => {
                    let amount = code.take_while_ref(|x| *x == b'+').count() + 1;
                    if amount > u8::MAX as usize {
                        return Err("An overflow occurred");
                    }
                    dynasm!(ops
                        ; add BYTE [a_current], amount as _
                        ; jo ->overflow
                    );
                },
                b'-' => {
                    let amount = code.take_while_ref(|x| *x == b'-').count() + 1;
                    if amount > u8::MAX as usize {
                        return Err("An overflow occurred");
                    }
                    dynasm!(ops
                        ; sub BYTE [a_current], amount as _
                        ; jo ->overflow
                    );
                },
                b',' => {
                    call_extern!(ops, State::getchar);
                    dynasm!(ops
                        ; cmp al, 0
                        ; jnz ->io_failure
                    );
                },
                b'.' => {
                    call_extern!(ops, State::putchar);
                    dynasm!(ops
                        ; cmp al, 0
                        ; jnz ->io_failure
                    );
                },
                b'[' => {
                    let first = code.peek() == Some(&b'-');
                    if first && code.peek() == Some(&b']') {
                        code.next();
                        code.next();
                        dynasm!(ops
                            ; mov BYTE [a_current], 0
                        );
                    } else {
                        let backward_label = ops.new_dynamic_label();
                        let forward_label  = ops.new_dynamic_label();
                        loops.push((backward_label, forward_label));
                        dynasm!(ops
                            ; cmp BYTE [a_current], 0
                            ; jz =>forward_label
                            ;=>backward_label
                        );
                    }
                },
                b']' => {
                    if let Some((backward_label, forward_label)) = loops.pop() {
                        dynasm!(ops
                            ; cmp BYTE [a_current], 0
                            ; jnz =>backward_label
                            ;=>forward_label
                        );
                    } else {
                        return Err("] without matching [");
                    }
                },
                _ => ()
            }
        }
        if loops.len() != 0 {
            return Err("[ without matching ]");
        }

        epilogue!(ops, 0);

        dynasm!(ops
            ;->overflow:
        );
        epilogue!(ops, 1);

        dynasm!(ops
            ;->io_failure:
        );
        epilogue!(ops, 2);

        let code = ops.finalize().unwrap();
        Ok(Program {
            code: code,
            start: start
        })
    }

    fn run(self, state: &mut State) -> Result<(), &'static str> {
        let f: extern "win64" fn(*mut State, *mut u8, *mut u8, *const u8) -> u8 = unsafe {
            mem::transmute(self.code.ptr(self.start))
        };
        let start = state.tape.as_mut_ptr();
        let end = unsafe { start.offset(TAPE_SIZE as isize) };
        let res = f(state, start, start, end);
        if res == 0 {
            Ok(())
        } else if res == 1 {
            Err("An overflow occurred")
        } else if res == 2 {
            Err("IO error")
        } else {
            panic!("Unknown error code");
        }
    }   
}

impl<'a> State<'a> {
    unsafe extern "win64" fn getchar(state: *mut State, cell: *mut u8) -> u8 {
        let state = &mut *state;
        let err = state.output.flush().is_err();
        (state.input.read_exact(slice::from_raw_parts_mut(cell, 1)).is_err() || err) as u8
    }

    unsafe extern "win64" fn putchar(state: *mut State, cell: *mut u8) -> u8 {
        let state = &mut *state;
        state.output.write_all(slice::from_raw_parts(cell, 1)).is_err() as u8
    }

    fn new(input: Box<BufRead + 'a>, output: Box<Write + 'a>) -> State<'a> {
        State {
            input: input,
            output: output,
            tape: [0; TAPE_SIZE]
        }
    }
}


fn main() {
    let mut args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Expected 1 argument, got {}", args.len());
        return;
    }
    let path = args.pop().unwrap();

    let mut f = if let Ok(f) = File::open(&path) { f } else {
        println!("Could not open file {}", path);
        return;
    };

    let mut buf = Vec::new();
    if let Err(_) = f.read_to_end(&mut buf) {
        println!("Failed to read from file");
        return;
    }

    let mut state = State::new(
        Box::new(BufReader::new(stdin())), 
        Box::new(BufWriter::new(stdout()))
    );
    let program = match Program::compile(&buf) {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    if let Err(e) = program.run(&mut state) {
        println!("{}", e);
        return;
    }
}
