use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::process;
use std::time::Instant;

// ASCII character codes
const ASC_TAB: u8 = 8;
const ASC_LF: u8 = 10;
const ASC_FF: u8 = 12;
const ASC_CR: u8 = 13;
const ASC_SPACE: u8 = 32;
const ASC_DOLLAR: u8 = 36;
const ASC_PERCENT: u8 = 37;
const ASC_PLUS: u8 = 43;
const ASC_MINUS: u8 = 45;
const ASC_SLASH: u8 = 47;

const ASC_0: u8 = 48;
const ASC_9: u8 = 57;
const ASC_A: u8 = 65;
const ASC_Z: u8 = 90;

const ASC_L: u8 = 76;
const ASC_S: u8 = 83;
const ASC_J: u8 = 74;
const ASC_T: u8 = 84;
const ASC_F: u8 = 70;
const ASC_K: u8 = 75;
const ASC_X: u8 = 88;
const ASC_C: u8 = 67;
const ASC_D: u8 = 68;
const ASC_G: u8 = 71;
const ASC_I: u8 = 73;
const ASC_P: u8 = 80;
const ASC_O: u8 = 79;
const ASC_N: u8 = 78;

// Memory configuration
const PROGSTART: usize = 401;
const WORDCOUNT: usize = 19900;
const LABVCOUNT: usize = 500;

// Instruction encoding
const FN_BITS: i16 = 8;
const FN_MASK: i16 = 255;
const F0_L: i16 = 0;
const F1_S: i16 = 1;
const F2_A: i16 = 2;
const F3_J: i16 = 3;
const F4_T: i16 = 4;
const F5_F: i16 = 5;
const F6_K: i16 = 6;
const F7_X: i16 = 7;
const FI_BIT: i16 = 1 << 3;
const FP_BIT: i16 = 1 << 4;
const FD_BIT: i16 = 1 << 5;

// K-codes (system calls)
const K01_START: i16 = 1;
const K11_SELECTINPUT: i16 = 11;
const K12_SELECTOUTPUT: i16 = 12;
const K13_RDCH: i16 = 13;
const K14_WRCH: i16 = 14;
const K16_INPUT: i16 = 16;
const K17_OUTPUT: i16 = 17;
const K30_STOP: i16 = 30;
const K31_LEVEL: i16 = 31;
const K32_LONGJUMP: i16 = 32;
const K40_APTOVEC: i16 = 40;
const K41_FINDOUTPUT: i16 = 41;
const K42_FINDINPUT: i16 = 42;
const K46_ENDREAD: i16 = 46;
const K47_ENDWRITE: i16 = 47;
const K60_WRITES: i16 = 60;
const K62_WRITEN: i16 = 62;
const K63_NEWLINE: i16 = 63;
const K64_NEWPAGE: i16 = 64;
const K66_PACKSTRING: i16 = 66;
const K67_UNPACKSTRING: i16 = 67;
const K68_WRITED: i16 = 68;
const K70_READN: i16 = 70;
const K71_TERMINATOR: i16 = 71;
const K75_WRITEHEX: i16 = 75;
const K76_WRITEF: i16 = 76;
const K77_WRITEOCT: i16 = 77;
const K85_GETBYTE: i16 = 85;
const K86_PUTBYTE: i16 = 86;
const K87_GETVEC: i16 = 87;
const K88_FREEVEC: i16 = 88;
const K90_CHANGECO: i16 = 90;

const ENDSTREAMCH: i16 = -1;
const BYTESPERWORD: usize = 2;

// Global state
struct BcplState {
    m: Vec<i16>,
    lomem: usize,
    himem: usize,
    heap_top: usize,
    free_list: Vec<(usize, usize)>,
    alloc_sizes: Vec<usize>,
    cis: usize,
    cos: usize,
    sysin: usize,
    sysprint: usize,
    cp: usize,
    ch: i16,
    files: Vec<Option<FileHandle>>,
    co_debug: bool,
    co_debug_last: Instant,
    co_debug_steps: u64,
}

enum FileHandle {
    Reader(BufReader<File>),
    Writer(BufWriter<File>),
    Stdin,
    Stdout,
}

impl BcplState {
    fn new() -> Self {
        let m = vec![0i16; WORDCOUNT];
        
        BcplState {
            m,
            lomem: 0,
            himem: WORDCOUNT - 1,
            heap_top: WORDCOUNT - 1,
            free_list: Vec::new(),
            alloc_sizes: vec![0; WORDCOUNT],
            cis: 1,
            cos: 2,
            sysin: 1,
            sysprint: 2,
            cp: 0,
            ch: 0,
            files: vec![None, Some(FileHandle::Stdin), Some(FileHandle::Stdout)],
            co_debug: false,
            co_debug_last: Instant::now(),
            co_debug_steps: 0,
        }
    }

    fn get_byte(&self, byte_idx: usize) -> u8 {
        let word_idx = byte_idx >> 1;
        let val = self.m[word_idx] as u16;
        if byte_idx & 1 != 0 {
            ((val >> 8) & 0xFF) as u8
        } else {
            (val & 0xFF) as u8
        }
    }

    fn set_byte(&mut self, byte_idx: usize, val: u8) {
        let word_idx = byte_idx >> 1;
        let word = self.m[word_idx] as u16;
        if byte_idx & 1 != 0 {
            self.m[word_idx] = ((word & 0x00FF) | ((val as u16) << 8)) as i16;
        } else {
            self.m[word_idx] = ((word & 0xFF00) | (val as u16)) as i16;
        }
    }

    fn cstr(&self, s_ptr: usize) -> String {
        let byte_idx = s_ptr * 2;
        let len = self.get_byte(byte_idx) as usize;
        let mut result = String::with_capacity(len);
        for i in 0..len {
            result.push(self.get_byte(byte_idx + 1 + i) as char);
        }
        result
    }

    fn openfile(&mut self, filename: &str, mode: &str) -> usize {
        if filename.eq_ignore_ascii_case("SYSIN") {
            return self.sysin;
        }
        if filename.eq_ignore_ascii_case("SYSPRINT") {
            return self.sysprint;
        }

        let handle = if mode == "r" {
            if let Ok(file) = File::open(filename) {
                Some(FileHandle::Reader(BufReader::new(file)))
            } else if let Ok(file) = File::open(filename.to_lowercase()) {
                Some(FileHandle::Reader(BufReader::new(file)))
            } else {
                return 0;
            }
        } else {
            if let Ok(file) = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(filename)
            {
                Some(FileHandle::Writer(BufWriter::new(file)))
            } else {
                return 0;
            }
        };

        if let Some(h) = handle {
            self.files.push(Some(h));
            self.files.len() - 1
        } else {
            0
        }
    }

    fn findinput(&mut self, fn_ptr: usize) -> usize {
        let filename = self.cstr(fn_ptr);
        self.openfile(&filename, "r")
    }

    fn findoutput(&mut self, fn_ptr: usize) -> usize {
        let filename = self.cstr(fn_ptr);
        self.openfile(&filename, "w")
    }

    fn endread(&mut self) {
        if self.cis != self.sysin && self.cis < self.files.len() {
            self.files[self.cis] = None;
            self.cis = self.sysin;
        }
    }

    fn endwrite(&mut self) {
        if self.cos != self.sysprint && self.cos < self.files.len() {
            if let Some(FileHandle::Writer(w)) = &mut self.files[self.cos] {
                let _ = w.flush();
            }
            self.files[self.cos] = None;
            self.cos = self.sysprint;
        }
    }

    fn rdch(&mut self) -> i16 {
        if self.cis >= self.files.len() {
            return ENDSTREAMCH;
        }

        let mut buf = [0u8; 1];
        let result = match &mut self.files[self.cis] {
            Some(FileHandle::Reader(reader)) => reader.read(&mut buf),
            Some(FileHandle::Stdin) => io::stdin().read(&mut buf),
            _ => return ENDSTREAMCH,
        };

        match result {
            Ok(0) => ENDSTREAMCH,
            Ok(_) => {
                let c = buf[0];
                if c == ASC_CR { ASC_LF as i16 } else { c as i16 }
            }
            Err(_) => ENDSTREAMCH,
        }
    }

    fn wrch(&mut self, c: i16) {
        if c == ASC_LF as i16 {
            self.newline();
        } else {
            let buf = [c as u8];
            match &mut self.files[self.cos] {
                Some(FileHandle::Writer(writer)) => {
                    let _ = writer.write(&buf);
                }
                Some(FileHandle::Stdout) => {
                    let _ = io::stdout().write(&buf);
                }
                _ => {}
            }
        }
    }

    fn newline(&mut self) {
        match &mut self.files[self.cos] {
            Some(FileHandle::Writer(writer)) => {
                let _ = writer.write(b"\n");
            }
            Some(FileHandle::Stdout) => {
                let _ = io::stdout().write(b"\n");
                let _ = io::stdout().flush();
            }
            _ => {}
        }
    }

    fn writes(&mut self, s_ptr: usize) {
        let byte_idx = s_ptr * 2;
        let len = self.get_byte(byte_idx) as usize;
        for i in 0..len {
            self.wrch(self.get_byte(byte_idx + 1 + i) as i16);
        }
    }

    fn writed(&mut self, n: i16, d: i16) {
        let s = format!("{}", n);
        let padding = if d as usize > s.len() {
            d as usize - s.len()
        } else {
            0
        };
        for _ in 0..padding {
            self.wrch(ASC_SPACE as i16);
        }
        for c in s.bytes() {
            self.wrch(c as i16);
        }
    }

    fn writen(&mut self, n: i16) {
        self.writed(n, 0);
    }

    fn readn(&mut self) -> i16 {
        let mut sum = 0i16;
        let mut neg = false;

        loop {
            self.ch = self.rdch();
            if self.ch != ASC_SPACE as i16
                && self.ch != ASC_LF as i16
                && self.ch != ASC_TAB as i16
            {
                break;
            }
        }

        if self.ch == ASC_MINUS as i16 {
            neg = true;
            self.ch = self.rdch();
        } else if self.ch == ASC_PLUS as i16 {
            self.ch = self.rdch();
        }

        while self.ch >= ASC_0 as i16 && self.ch <= ASC_9 as i16 {
            sum = sum.wrapping_mul(10).wrapping_add(self.ch - ASC_0 as i16);
            self.ch = self.rdch();
        }

        self.m[K71_TERMINATOR as usize] = self.ch;
        if neg { -sum } else { sum }
    }

    fn writeoct(&mut self, n: u16, d: i16) {
        if d > 1 {
            self.writeoct(n >> 3, d - 1);
        }
        let digit = (n & 7) as u8;
        self.wrch((b'0' + digit) as i16);
    }

    fn writehex(&mut self, n: u16, d: i16) {
        if d > 1 {
            self.writehex(n >> 4, d - 1);
        }
        let digit = (n & 15) as u8;
        let c = if digit < 10 {
            b'0' + digit
        } else {
            b'A' + digit - 10
        };
        self.wrch(c as i16);
    }

    fn getvec(&mut self, words: usize, sp: u16) -> i16 {
        if words == 0 || words >= WORDCOUNT {
            return 0;
        }

        if let Some((idx, (addr, size))) = self
            .free_list
            .iter()
            .enumerate()
            .find(|(_, (_, size))| *size >= words)
        {
            let (addr, size) = (*addr, *size);
            if size == words {
                self.free_list.swap_remove(idx);
            } else {
                self.free_list[idx] = (addr + words, size - words);
            }
            self.alloc_sizes[addr] = words;
            return addr as i16;
        }

        if self.heap_top < words {
            return 0;
        }

        let start = self.heap_top + 1 - words;
        if start <= sp as usize + 1 {
            return 0;
        }

        self.heap_top = start - 1;
        self.alloc_sizes[start] = words;
        start as i16
    }

    fn freevec(&mut self, addr: usize) -> i16 {
        if addr >= self.alloc_sizes.len() {
            return 0;
        }
        let size = self.alloc_sizes[addr];
        if size == 0 {
            return 0;
        }

        self.alloc_sizes[addr] = 0;
        self.free_list.push((addr, size));
        self.free_list.sort_by_key(|(a, _)| *a);

        let mut merged: Vec<(usize, usize)> = Vec::with_capacity(self.free_list.len());
        for (a, s) in self.free_list.drain(..) {
            if let Some((last_a, last_s)) = merged.last_mut() {
                if *last_a + *last_s == a {
                    *last_s += s;
                    continue;
                }
            }
            merged.push((a, s));
        }
        self.free_list = merged;
        1
    }

    fn decval(&self, c: u8) -> i16 {
        if c >= ASC_0 && c <= ASC_9 {
            (c - ASC_0) as i16
        } else if c >= ASC_A && c <= ASC_Z {
            (c - ASC_A + 10) as i16
        } else {
            0
        }
    }

    fn writef(&mut self, v_ptr: usize) {
        let fmt_ptr = self.m[v_ptr] as usize;
        let mut v_idx = v_ptr + 1;
        let byte_idx = fmt_ptr * 2;
        let len = self.get_byte(byte_idx) as usize;
        let mut ss = 1;

        while ss <= len {
            let c = self.get_byte(byte_idx + ss);
            ss += 1;
            if c != ASC_PERCENT {
                self.wrch(c as i16);
            } else {
                let c = self.get_byte(byte_idx + ss);
                ss += 1;
                match c {
                    b'S' => {
                        self.writes(self.m[v_idx] as usize);
                        v_idx += 1;
                    }
                    b'C' => {
                        self.wrch(self.m[v_idx]);
                        v_idx += 1;
                    }
                    b'O' => {
                        let val = self.m[v_idx] as u16;
                        let d = self.decval(self.get_byte(byte_idx + ss));
                        ss += 1;
                        self.writeoct(val, d);
                        v_idx += 1;
                    }
                    b'X' => {
                        let val = self.m[v_idx] as u16;
                        let d = self.decval(self.get_byte(byte_idx + ss));
                        ss += 1;
                        self.writehex(val, d);
                        v_idx += 1;
                    }
                    b'I' => {
                        let val = self.m[v_idx];
                        let d = self.decval(self.get_byte(byte_idx + ss));
                        ss += 1;
                        self.writed(val, d);
                        v_idx += 1;
                    }
                    b'N' => {
                        self.writen(self.m[v_idx]);
                        v_idx += 1;
                    }
                    _ => {
                        self.wrch(c as i16);
                    }
                }
            }
        }
    }

    fn packstring(&mut self, v_ptr: usize, s_ptr: usize) -> i16 {
        let len = self.m[v_ptr] as usize;
        let n = len / BYTESPERWORD;
        
        self.m[s_ptr + n] = 0;
        
        for i in 0..=len {
            self.set_byte(s_ptr * 2 + i, (self.m[v_ptr + i] & 0xFF) as u8);
        }
        
        n as i16
    }

    fn unpackstring(&mut self, s_ptr: usize, v_ptr: usize) {
        let byte_idx = s_ptr * 2;
        let len = self.get_byte(byte_idx) as usize;
        
        for i in 0..=len {
            self.m[v_ptr + i] = self.get_byte(byte_idx + i) as i16;
        }
    }

    fn stw(&mut self, w: i16) {
        self.m[self.lomem] = w;
        self.lomem += 1;
        self.cp = 0;
    }

    fn stc(&mut self, c: i16) {
        if self.cp == 0 {
            self.stw(0);
        }
        let byte_addr = (self.lomem - 1) * 2 + self.cp;
        self.set_byte(byte_addr, c as u8);
        self.cp += 1;
        if self.cp == BYTESPERWORD {
            self.cp = 0;
        }
    }

    fn rch(&mut self) {
        self.ch = self.rdch();
        while self.ch == ASC_SLASH as i16 {
            loop {
                self.ch = self.rdch();
                if self.ch == ASC_LF as i16 || self.ch == ENDSTREAMCH {
                    break;
                }
            }
            while self.ch == ASC_LF as i16 {
                self.ch = self.rdch();
            }
        }
    }

    fn rdn(&mut self) -> i16 {
        let mut sum = 0i16;
        let neg = self.ch == ASC_MINUS as i16;
        if neg {
            self.rch();
        }
        while self.ch >= ASC_0 as i16 && self.ch <= ASC_9 as i16 {
            sum = sum.wrapping_mul(10).wrapping_add(self.ch - ASC_0 as i16);
            self.rch();
        }
        if neg { -sum } else { sum }
    }

    fn labref(&mut self, n: i16, a: usize) {
        let labv_offset = WORDCOUNT - LABVCOUNT;
        let mut k = self.m[labv_offset + n as usize];
        if k < 0 {
            k = -k;
        } else {
            self.m[labv_offset + n as usize] = a as i16;
        }
        self.m[a] = self.m[a].wrapping_add(k);
    }

    fn halt(&mut self, msg: &str, n: i16) -> ! {
        self.cos = self.sysprint;
        let msg_str = if n != 0 {
            format!("{} #{}\n", msg, n)
        } else {
            format!("{}\n", msg)
        };
        match &mut self.files[self.cos] {
            Some(FileHandle::Writer(w)) => {
                let _ = w.write(msg_str.as_bytes());
                let _ = w.flush();
            }
            Some(FileHandle::Stdout) => {
                let _ = io::stdout().write(msg_str.as_bytes());
                let _ = io::stdout().flush();
            }
            _ => {}
        }
        process::exit(1);
    }

    fn assemble(&mut self) {
        let labv_offset = WORDCOUNT - LABVCOUNT;
        
        // Clear labels
        for i in 0..LABVCOUNT {
            self.m[labv_offset + i] = 0;
        }
        self.cp = 0;

        self.rch();

        loop {
            // Check for label definition (digit)
            if self.ch >= ASC_0 as i16 && self.ch <= ASC_9 as i16 {
                let n = self.rdn();
                let mut k = self.m[labv_offset + n as usize];
                if k < 0 {
                    self.halt("DUPLICATE LABEL", n);
                }
                while k > 0 {
                    let tmp = self.m[k as usize];
                    self.m[k as usize] = self.lomem as i16;
                    k = tmp;
                }
                self.m[labv_offset + n as usize] = -(self.lomem as i16);
                self.cp = 0;
                continue;
            }

            match self.ch as u8 {
                b'$' | ASC_SPACE | ASC_LF => {
                    self.rch();
                    continue;
                }
                b'L' => self.process_instruction(F0_L),
                b'S' => self.process_instruction(F1_S),
                b'A' => self.process_instruction(F2_A),
                b'J' => self.process_instruction(F3_J),
                b'T' => self.process_instruction(F4_T),
                b'F' => self.process_instruction(F5_F),
                b'K' => self.process_instruction(F6_K),
                b'X' => self.process_instruction(F7_X),
                b'C' => {
                    self.rch();
                    let val = self.rdn();
                    self.stc(val);
                    continue;
                }
                b'D' => {
                    self.rch();
                    if self.ch == b'L' as i16 {
                        self.rch();
                        self.stw(0);
                        let n = self.rdn();
                        let addr = self.lomem - 1;
                        self.labref(n, addr);
                    } else {
                        let val = self.rdn();
                        self.stw(val);
                    }
                    continue;
                }
                b'G' => {
                    self.rch();
                    let n = self.rdn();
                    if self.ch != b'L' as i16 {
                        self.halt("BAD CODE AT P", self.lomem as i16);
                    }
                    self.rch();
                    self.m[n as usize] = 0;
                    let lab = self.rdn();
                    self.labref(lab, n as usize);
                    continue;
                }
                b'Z' => {
                    for n in 0..LABVCOUNT {
                        if self.m[labv_offset + n] > 0 {
                            self.halt("UNSET LABEL", n as i16);
                        }
                    }
                    // Clear and restart
                    for i in 0..LABVCOUNT {
                        self.m[labv_offset + i] = 0;
                    }
                    self.cp = 0;
                    self.rch();
                    continue;
                }
                _ => {
                    if self.ch == ENDSTREAMCH {
                        return;
                    }
                    self.halt("BAD CH", self.ch);
                }
            }
        }
    }

    fn process_instruction(&mut self, mut n: i16) {
        self.rch();
        if self.ch == b'I' as i16 {
            n |= FI_BIT;
            self.rch();
        }
        if self.ch == b'P' as i16 {
            n |= FP_BIT;
            self.rch();
        }
        if self.ch == b'G' as i16 {
            self.rch();
        }
        
        if self.ch == b'L' as i16 {
            self.rch();
            self.stw(n | FD_BIT);
            self.stw(0);
            let lab = self.rdn();
            let addr = self.lomem - 1;
            self.labref(lab, addr);
        } else {
            let d = self.rdn();
            if (d & FN_MASK) == d {
                self.stw(n | (d << FN_BITS));
            } else {
                self.stw(n | FD_BIT);
                self.stw(d);
            }
        }
    }

    fn interpret(&mut self) -> i16 {
        let mut pc: u16 = PROGSTART as u16;
        let mut sp: u16 = self.lomem as u16;
        let mut a: i16 = 0;
        let mut b: i16 = 0;

        loop {
            if self.co_debug {
                self.co_debug_steps = self.co_debug_steps.wrapping_add(1);
                if self.co_debug_steps % 1_000_000 == 0 {
                    if self.co_debug_last.elapsed().as_millis() >= 500 {
                        let currco = self.m.get(500).copied().unwrap_or(0);
                        eprintln!(
                            "TRACE: pc={} sp={} currco={} steps={}",
                            pc, sp, currco, self.co_debug_steps
                        );
                        self.co_debug_last = Instant::now();
                    }
                }
            }
            let w: u16 = self.m[pc as usize] as u16;
            pc = pc.wrapping_add(1);

            // d is unsigned just like in C: register word d
            let mut d: u16 = if w & (FD_BIT as u16) != 0 {
                let val = self.m[pc as usize];
                pc = pc.wrapping_add(1);
                val as u16
            } else {
                w >> FN_BITS
            };

            if w & (FP_BIT as u16) != 0 {
                d = d.wrapping_add(sp);
            }
            if w & (FI_BIT as u16) != 0 {
                d = self.m[d as usize] as u16;
            }

            match w & F7_X as u16 {
                0 => { // F0_L
                    b = a;
                    a = d as i16;
                }
                1 => { // F1_S
                    self.m[d as usize] = a;
                }
                2 => { // F2_A
                    a = a.wrapping_add(d as i16);
                }
                3 => { // F3_J
                    pc = d;
                }
                4 => { // F4_T
                    if a != 0 {
                        pc = d;
                    }
                }
                5 => { // F5_F
                    if a == 0 {
                        pc = d;
                    }
                }
                6 => { // F6_K
                    let d_addr = d.wrapping_add(sp);
                    if a < PROGSTART as i16 {
                        let v_ptr = (d_addr + 2) as usize;
                        match a {
                            K01_START => {}
                            K11_SELECTINPUT => self.cis = self.m[v_ptr] as usize,
                            K12_SELECTOUTPUT => self.cos = self.m[v_ptr] as usize,
                            K13_RDCH => a = self.rdch(),
                            K14_WRCH => self.wrch(self.m[v_ptr]),
                            K16_INPUT => a = self.cis as i16,
                            K17_OUTPUT => a = self.cos as i16,
                            K30_STOP => return self.m[v_ptr],
                            K31_LEVEL => a = sp as i16,
                            K32_LONGJUMP => {
                                sp = self.m[v_ptr] as u16;
                                pc = self.m[v_ptr + 1] as u16;
                            }
                            K40_APTOVEC => {
                                let b_addr = d_addr.wrapping_add(self.m[v_ptr + 1] as u16).wrapping_add(1);
                                if self.co_debug {
                                    eprintln!(
                                        "APTOVEC: sp={} d_addr={} argc={} b_addr={} pc={}",
                                        sp,
                                        d_addr,
                                        self.m[v_ptr + 1],
                                        b_addr,
                                        pc
                                    );
                                }
                                self.m[b_addr as usize] = sp as i16;
                                self.m[b_addr as usize + 1] = pc as i16;
                                self.m[b_addr as usize + 2] = d_addr as i16;  // BUG FIX: was 'd', should be 'd_addr'
                                self.m[b_addr as usize + 3] = self.m[v_ptr + 1];
                                sp = b_addr;
                                pc = self.m[v_ptr] as u16;
                            }
                            K41_FINDOUTPUT => a = self.findoutput(self.m[v_ptr] as usize) as i16,
                            K42_FINDINPUT => a = self.findinput(self.m[v_ptr] as usize) as i16,
                            K46_ENDREAD => self.endread(),
                            K47_ENDWRITE => self.endwrite(),
                            K60_WRITES => self.writes(self.m[v_ptr] as usize),
                            K62_WRITEN => self.writen(self.m[v_ptr]),
                            K63_NEWLINE => self.newline(),
                            K64_NEWPAGE => self.wrch(ASC_FF as i16),
                            K66_PACKSTRING => {
                                a = self.packstring(self.m[v_ptr] as usize, self.m[v_ptr + 1] as usize)
                            }
                            K67_UNPACKSTRING => {
                                self.unpackstring(self.m[v_ptr] as usize, self.m[v_ptr + 1] as usize)
                            }
                            K68_WRITED => self.writed(self.m[v_ptr], self.m[v_ptr + 1]),
                            K70_READN => a = self.readn(),
                            K75_WRITEHEX => self.writehex(self.m[v_ptr] as u16, self.m[v_ptr + 1]),
                            K76_WRITEF => self.writef(v_ptr),
                            K77_WRITEOCT => self.writeoct(self.m[v_ptr] as u16, self.m[v_ptr + 1]),
                            K85_GETBYTE => {
                                let base = (self.m[v_ptr] as u16 as usize) * 2;
                                let offset = self.m[v_ptr + 1] as usize;
                                a = self.get_byte(base + offset) as i16;
                            }
                            K86_PUTBYTE => {
                                let base = (self.m[v_ptr] as u16 as usize) * 2;
                                let offset = self.m[v_ptr + 1] as usize;
                                self.set_byte(base + offset, self.m[v_ptr + 2] as u8);
                            }
                            K87_GETVEC => {
                                let words = self.m[v_ptr] as u16 as usize;
                                a = self.getvec(words, sp);
                            }
                            K88_FREEVEC => {
                                let addr = self.m[v_ptr] as u16 as usize;
                                a = self.freevec(addr);
                            }
                            K90_CHANGECO => {
                                // Changeco(A, Cptr, CurrcoAddr) with saved sp/pc
                                let arg = self.m[v_ptr];
                                let cptr = self.m[v_ptr + 1] as u16 as usize;
                                let currco_addr = self.m[v_ptr + 2] as u16 as usize;

                                let currco = self.m[currco_addr] as u16 as usize;
                                if self.co_debug {
                                    eprintln!(
                                        "CHANGECO enter: arg={} currco_addr={} currco={} -> cptr={} sp={} pc={}",
                                        arg, currco_addr, currco, cptr, sp, pc
                                    );
                                    if cptr < self.m.len().saturating_sub(6) {
                                        eprintln!(
                                            "CHANGECO cptr fields: sp={} pc={} parent={} next={} f={} size={} self={}",
                                            self.m[cptr],
                                            self.m[cptr + 1],
                                            self.m[cptr + 2],
                                            self.m[cptr + 3],
                                            self.m[cptr + 4],
                                            self.m[cptr + 5],
                                            self.m[cptr + 6]
                                        );
                                    }
                                }
                                if currco != 0 {
                                    self.m[currco] = sp as i16;
                                    self.m[currco + 1] = pc as i16;
                                }

                                self.m[currco_addr] = cptr as i16;
                                sp = self.m[cptr] as u16;
                                pc = self.m[cptr + 1] as u16;
                                a = arg;
                                if self.co_debug {
                                    eprintln!(
                                        "CHANGECO exit: currco_addr={} currco={} sp={} pc={}",
                                        currco_addr, cptr, sp, pc
                                    );
                                }
                            }
                            _ => self.halt("UNKNOWN CALL", a),
                        }
                    } else {
                        self.m[d_addr as usize] = sp as i16;
                        self.m[d_addr as usize + 1] = pc as i16;
                        sp = d_addr;
                        pc = a as u16;
                    }
                }
                7 => { // F7_X
                    match d {
                        1 => a = self.m[a as u16 as usize],
                        2 => a = -a,
                        3 => a = !a,
                        4 => {
                            pc = self.m[sp as usize + 1] as u16;
                            sp = self.m[sp as usize] as u16;
                        }
                        5 => a = a.wrapping_mul(b),
                        6 => {
                            if a != 0 {
                                a = b / a;
                            }
                        }
                        7 => {
                            if a != 0 {
                                a = b % a;
                            }
                        }
                        8 => a = b.wrapping_add(a),
                        9 => a = b.wrapping_sub(a),
                        10 => a = if b == a { -1 } else { 0 },
                        11 => a = if b != a { -1 } else { 0 },
                        12 => a = if b < a { -1 } else { 0 },
                        13 => a = if b >= a { -1 } else { 0 },
                        14 => a = if b > a { -1 } else { 0 },
                        15 => a = if b <= a { -1 } else { 0 },
                        16 => a = b << a,
                        17 => a = ((b as u16) >> a) as i16,
                        18 => a = b & a,
                        19 => a = b | a,
                        20 => a = b ^ a,
                        21 => a = b ^ !a,
                        22 => return 0,
                        23 => {
                            let mut v_idx = pc as usize;
                            let mut count = self.m[v_idx];
                            v_idx += 1;
                            pc = self.m[v_idx] as u16;
                            v_idx += 1;
                            
                            while count > 0 {
                                if a == self.m[v_idx] {
                                    pc = self.m[v_idx + 1] as u16;
                                    break;
                                }
                                v_idx += 2;
                                count -= 1;
                            }
                        }
                        _ => {
                            self.halt("UNKNOWN EXEC", d as i16);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn loadcode(&mut self, filename: &str) -> bool {
        let f = self.openfile(filename, "r");
        if f != 0 {
            self.cis = f;
            self.assemble();
            self.endread();
            true
        } else {
            false
        }
    }

    fn init(&mut self) {
        for i in 0..PROGSTART {
            self.m[i] = i as i16;
        }
        self.lomem = PROGSTART;
        self.heap_top = WORDCOUNT - 1;
        self.free_list.clear();
        self.alloc_sizes.fill(0);
        
        self.stw(F0_L | FI_BIT | (K01_START << FN_BITS));
        self.stw(F6_K | (2 << FN_BITS));
        self.stw(F7_X | (22 << FN_BITS));
    }

    fn pipeinput(&mut self, filename: &str) {
        let f = self.openfile(filename, "r");
        if f == 0 {
            self.halt("NO INPUT", 0);
        }
        self.cis = f;
        self.sysin = f;
    }

    fn pipeoutput(&mut self, filename: &str) {
        let f = self.openfile(filename, "w");
        if f == 0 {
            self.halt("NO OUTPUT", 0);
        }
        self.cos = f;
        self.sysprint = f;
    }
}

fn main() {
    let mut state = BcplState::new();
    state.init();
    state.co_debug = env::var("BCPL_CO_DEBUG")
        .ok()
        .map(|v| v != "0")
        .unwrap_or(false);

    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() {
        eprintln!("USAGE: icint ICFILE [...] [-iINPUT] [-oOUTPUT]");
        process::exit(0);
    }

    for arg in args {
        if arg.starts_with('-') {
            if arg.starts_with("-i") {
                state.pipeinput(&arg[2..]);
            } else if arg.starts_with("-o") {
                state.pipeoutput(&arg[2..]);
            } else {
                state.halt("INVALID OPTION", 0);
            }
        } else {
            if !state.loadcode(&arg) {
                state.halt("NO ICFILE", 0);
            }
        }
    }

    state.interpret();
}
