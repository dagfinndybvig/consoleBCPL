use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::process;

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
struct WatchRegion {
    start: usize,
    end: usize,
    label: String,
    triggered: bool,
    // Snapshot of the first N words of the region at allocation time
    snapshot: Vec<i16>,
    // Optionally recorded first-write info: (idx, pc, sp, a, old, new)
    first_write: Option<(usize, u16, u16, i16, i16, i16)>,
}

struct InstrRec {
    pc: u16,
    w: u16,
    d: u16,
    a: i16,
    sp: u16,
}

struct WriteRec {
    idx: usize,
    pc: u16,
    sp: u16,
    a: i16,
    old: i16,
    new: i16,
}

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
    watch_regions: Vec<WatchRegion>,
    // Last instruction context for write-site reporting
    last_pc: u16,
    last_sp: u16,
    last_a: i16,
    instr_count: u64,
    instr_history: std::collections::VecDeque<InstrRec>,
    history_size: usize,
    // Recent write provenance buffer (only used when co_debug is enabled)
    recent_writes: std::collections::VecDeque<WriteRec>,
    recent_writes_limit: usize,
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
            watch_regions: Vec::new(),
            last_pc: 0,
            last_sp: 0,
            last_a: 0,
            instr_count: 0,
            instr_history: std::collections::VecDeque::with_capacity(512),
            history_size: 512,
            recent_writes: std::collections::VecDeque::with_capacity(1024),
            recent_writes_limit: 1024,
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
        let old_word = self.m[word_idx];
        let word = old_word as u16;
        if byte_idx & 1 != 0 {
            self.m[word_idx] = ((word & 0x00FF) | ((val as u16) << 8)) as i16;
        } else {
            self.m[word_idx] = ((word & 0xFF00) | (val as u16)) as i16;
        }
        // Check whether this write touched any watched region
        let new_word = self.m[word_idx];
        if new_word != old_word {
            self.check_write_index(word_idx, self.last_pc, self.last_sp, self.last_a);
            self.record_write(word_idx, self.last_pc, self.last_sp, self.last_a, old_word, new_word);
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
            // Zero the returned region to avoid leaking stale data into newly-created
            // coroutine control blocks or other sensitive structures.
            for i in addr..(addr + words) {
                let old = self.m[i];
                self.m[i] = 0;
                if self.co_debug && old != self.m[i] {
                    self.check_write_index(i, self.last_pc, self.last_sp, self.last_a);
                    self.record_write(i, self.last_pc, self.last_sp, self.last_a, old, self.m[i]);
                }
            }
            if self.co_debug {
                eprintln!("GETVEC: reused free block zeroed {}..{}", addr, addr + words - 1);
                self.dump_free_state("after-getvec-reuse");
            }
            // If this looks like a coroutine control block allocation, add a watch
            if words >= 7 {
                // Check for overlap with existing watch regions
                for wr in &self.watch_regions {
                    if wr.start <= addr + words - 1 && wr.end >= addr {
                        eprintln!("WARNING: GETVEC reuse {}..{} overlaps existing watch {} {}..{}", addr, addr + words - 1, wr.label, wr.start, wr.end);
                        let start = if addr > 40 { addr - 40 } else { 0 };
                        let end = (addr + words - 1 + 40).min(self.m.len());
                        eprintln!("MEM around overlap {}..{} = {:?}", start, end, &self.m[start..end]);
                        self.dump_free_state("overlap-on-getvec-reuse");
                    }
                }
                self.add_watch_region(addr, addr + words - 1, &format!("alloc@{}", addr));
            }
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
        // Zero newly allocated words to avoid garbage in control blocks / stack
        for i in start..(start + words) {
            let old = self.m[i];
            self.m[i] = 0;
            if self.co_debug && old != self.m[i] {
                self.check_write_index(i, self.last_pc, self.last_sp, self.last_a);
                self.record_write(i, self.last_pc, self.last_sp, self.last_a, old, self.m[i]);
            }
        }
        if self.co_debug {
            eprintln!("GETVEC: allocated and zeroed {}..{}", start, start + words - 1);
            self.dump_free_state("after-getvec-alloc");
        }
        if words >= 7 {
            self.add_watch_region(start, start + words - 1, &format!("alloc@{}", start));
        }
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
        // If coroutine debugging is enabled, expose the free-list and
        // a compact view of active allocations to help diagnose
        // control-block corruption that may be caused by allocator reuse.
        if self.co_debug {
            self.dump_free_state("after-freevec");
        }

        // Sanity: ensure no free block overlaps any watched region or existing allocation
        for (a, s) in &self.free_list {
            let free_start = *a;
            let free_end = a + s - 1;
            for wr in &self.watch_regions {
                if (wr.start <= free_end && wr.end >= free_start) {
                    eprintln!("ALLOC/FREE OVERLAP detected: free {}..{} intersects watch {} {}..{}", free_start, free_end, wr.label, wr.start, wr.end);
                    eprintln!("watch snapshot: {:?}", wr.snapshot);
                    let start = if free_start > 40 { free_start - 40 } else { 0 };
                    let end = (free_end + 40).min(self.m.len());
                    eprintln!("MEM around overlap {}..{} = {:?}", start, end, &self.m[start..end]);
                    self.scan_possible_coroutines(start, end);
                    self.halt("ALLOC/FREE OVERLAP", 0);
                }
            }
            // Also check if there is an active allocation that starts inside this free block
            for (i, &sz) in self.alloc_sizes.iter().enumerate() {
                if sz != 0 {
                    if i >= free_start && i <= free_end {
                        eprintln!("ALLOC/FREE INCONSISTENCY: free {}..{} contains alloc at {} size={}", free_start, free_end, i, sz);
                        let start = if free_start > 40 { free_start - 40 } else { 0 };
                        let end = (free_end + 40).min(self.m.len());
                        eprintln!("MEM around inconsistency {}..{} = {:?}", start, end, &self.m[start..end]);
                        self.halt("ALLOC/FREE INCONSISTENCY", 0);
                    }
                }
            }
        }
        1
    }

    // Diagnostic helpers -------------------------------------------------

    fn dump_free_state(&self, label: &str) {
        if !self.co_debug { return; }
        eprintln!("FREE_STATE {}: free_list={:?}", label, self.free_list);
        // Show up to 40 non-zero allocation entries (addr, size)
        let mut entries: Vec<(usize, usize)> = Vec::new();
        for (i, &sz) in self.alloc_sizes.iter().enumerate() {
            if sz != 0 {
                entries.push((i, sz));
                if entries.len() >= 40 { break; }
            }
        }
        eprintln!("ALLOC_SIZES nonzero (up to 40): {:?}", entries);
    }

    fn dump_instr_history(&self) {
        if !self.co_debug { return; }
        eprintln!("--- INSTR HISTORY (last {}) ---", self.instr_history.len());
        for rec in &self.instr_history {
            eprintln!("pc={} w={} d={} a={} sp={}", rec.pc, rec.w, rec.d, rec.a, rec.sp);
        }
        eprintln!("--- END INSTR HISTORY ---");
    }

    fn record_write(&mut self, idx: usize, pc: u16, sp: u16, a: i16, old: i16, new: i16) {
        if !self.co_debug { return; }
        let wr = WriteRec { idx, pc, sp, a, old, new };
        self.recent_writes.push_back(wr);
        if self.recent_writes.len() > self.recent_writes_limit {
            self.recent_writes.pop_front();
        }
    }

    fn dump_recent_writes(&self, start: usize, end: usize) {
        if !self.co_debug { return; }
        eprintln!("--- RECENT WRITES in {}..{} (last {}) ---", start, end, self.recent_writes.len());
        for wr in &self.recent_writes {
            if wr.idx >= start && wr.idx <= end {
                eprintln!("WRITE idx={} pc={} sp={} a={} old={} new={}", wr.idx, wr.pc, wr.sp, wr.a, wr.old, wr.new);
            }
        }
        eprintln!("--- END RECENT WRITES ---");
    }

    fn decode_instr(&self, rec: &InstrRec) -> String {
        let w = rec.w;
        let fn_code = (w & (F7_X as u16)) as u16;
        let mut s = format!("pc={} w={} d={} a={} sp={}", rec.pc, rec.w, rec.d, rec.a, rec.sp);
        match fn_code {
            0 => s = format!("{} => L: load literal d={} -> a", s, rec.d),
            1 => s = format!("{} => S: store a -> [d]", s),
            2 => s = format!("{} => A: add d to a -> a", s),
            3 => s = format!("{} => J: jump pc := d", s),
            4 => s = format!("{} => T: if a!=0 pc := d ; (conditional)", s),
            5 => s = format!("{} => F: if a==0 pc := d ; (conditional)", s),
            6 => {
                // K ops: need to show d_addr and potential vector ptr
                let d_addr = rec.d.wrapping_add(rec.sp);
                if rec.a < PROGSTART as i16 {
                    s = format!("{} => K: syscall a={} v_ptr_calc=(d_addr+2)={} d_addr={} (syscall)", s, rec.a, d_addr + 2, d_addr);
                } else {
                    s = format!("{} => K: frame call variant a={} d_addr={} (frame op)", s, rec.a, d_addr);
                }
            }
            7 => s = format!("{} => X: extended op d={} (alu/other)", s, rec.d),
            _ => s = format!("{} => UNKNOWN_FN", s),
        }
        s
    }

    fn dump_decoded_history(&self, n: usize) {
        if !self.co_debug { return; }
        eprintln!("--- DECODED HISTORY (last {}) ---", n);
        let len = self.instr_history.len();
        let start = if len > n { len - n } else { 0 };
        for rec in self.instr_history.iter().skip(start) {
            eprintln!("  {}", self.decode_instr(rec));
        }
        eprintln!("--- END DECODED HISTORY ---");
    }

    fn add_watch_region(&mut self, start: usize, end: usize, label: &str) {
        if !self.co_debug { return; }
        // Keep watch only for reasonably-sized regions and avoid too many regions
        if end <= start { return; }
        if self.watch_regions.len() > 200 { return; }
        let len = end - start + 1;
        let snap_len = std::cmp::min(16, len);
        let snapshot = self.m[start..(start + snap_len)].to_vec();
        let wr = WatchRegion {
            start,
            end,
            label: label.to_string(),
            triggered: false,
            snapshot,
            first_write: None,
        };
        eprintln!("WATCH: adding region {} {}..{} (snapshot_len={})", label, start, end, snap_len);
        self.watch_regions.push(wr);
    }

    fn check_write_index(&mut self, idx: usize, pc: u16, sp: u16, a: i16) {
        if !self.co_debug { return; }
        for wr in self.watch_regions.iter_mut() {
            if wr.triggered { continue; }
            if idx >= wr.start && idx <= wr.end {
                // Determine snapshot offset and old value
                let offset = idx - wr.start;
                let old = if offset < wr.snapshot.len() { wr.snapshot[offset] } else { 0 };
                let new = self.m[idx];
                if new != old {
                    wr.triggered = true;
                    wr.first_write = Some((idx, pc, sp, a, old, new));
                    eprintln!("WATCH-HIT {} idx={} pc={} sp={} a={} old={} new={}", wr.label, idx, pc, sp, a, old, new);
                    // Also dump a small memory window around the hit for context
                    let start = if idx > 20 { idx - 20 } else { 0 };
                    let end = (idx + 20).min(self.m.len());
                    eprintln!("MEM around hit {}..{} = {:?}", start, end, &self.m[start..end]);
                }
            }
        }
    }

    fn scan_possible_coroutines(&self, start: usize, end: usize) {
        if !self.co_debug { return; }
        let mut found = 0;
        let s = start.min(self.m.len());
        let e = end.min(self.m.len());
        for off in (s..e).step_by(7) {
            if off + 6 < self.m.len() {
                let block = &self.m[off..off + 7.min(self.m.len()-off)];
                let alloc = self.alloc_sizes.get(off).copied().unwrap_or(0);
                // Heuristics: a coroutine control block usually has a saved pc
                // (C!1) that lies inside program memory, and a saved sp (C!0)
                // that is a valid word index.
                let maybe_pc = block.get(1).copied().unwrap_or(0) as usize;
                let maybe_sp = block.get(0).copied().unwrap_or(0) as usize;
                if (alloc >= 7) || (maybe_pc >= PROGSTART && maybe_pc < WORDCOUNT && maybe_sp >= PROGSTART && maybe_sp < WORDCOUNT) {
                    eprintln!("Possible coroutine @{} alloc={} block={:?}", off, alloc, block);
                    found += 1;
                    if found >= 40 { break; }
                }
            }
        }
        if found == 0 {
            eprintln!("scan_possible_coroutines: none found in {}..{}", s, e);
        }
    }

    fn is_plausible_coroutine(&self, v_ptr: usize) -> bool {
        // Quick sanity checks for a coroutine control block at v_ptr
        if v_ptr + 1 >= self.m.len() { return false; }
        let c0 = self.m[v_ptr] as usize; // saved sp
        let c1 = self.m[v_ptr + 1] as usize; // saved pc
        if c0 < PROGSTART || c0 >= WORDCOUNT { return false; }
        if c1 < PROGSTART || c1 >= WORDCOUNT { return false; }
        true
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
        
        let idx = s_ptr + n;
        let old = self.m[idx];
        self.m[idx] = 0;
        if self.co_debug && old != 0 {
            self.check_write_index(idx, self.last_pc, self.last_sp, self.last_a);
            self.record_write(idx, self.last_pc, self.last_sp, self.last_a, old, 0);
        }
        
        for i in 0..=len {
            self.set_byte(s_ptr * 2 + i, (self.m[v_ptr + i] & 0xFF) as u8);
        }
        
        n as i16
    }

    fn unpackstring(&mut self, s_ptr: usize, v_ptr: usize) {
        let byte_idx = s_ptr * 2;
        let len = self.get_byte(byte_idx) as usize;
        
        for i in 0..=len {
            let idx = v_ptr + i;
            let old = self.m[idx];
            self.m[idx] = self.get_byte(byte_idx + i) as i16;
            if self.co_debug && old != self.m[idx] {
                self.check_write_index(idx, self.last_pc, self.last_sp, self.last_a);
                self.record_write(idx, self.last_pc, self.last_sp, self.last_a, old, self.m[idx]);
            }
        }
    }

    fn stw(&mut self, w: i16) {
        let idx = self.lomem;
        let old = self.m[idx];
        self.m[idx] = w;
        if self.co_debug && self.m[idx] != old {
            self.check_write_index(idx, self.last_pc, self.last_sp, self.last_a);
            self.record_write(idx, self.last_pc, self.last_sp, self.last_a, old, self.m[idx]);
        }
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
        // If coroutine debugging is enabled, prepend a brief debug line
        // into the output so post-mortem logs contain additional context
        // even if stderr is not captured.
        let mut msg_str = if n != 0 {
            format!("{} #{}\n", msg, n)
        } else {
            format!("{}\n", msg)
        };
        if self.co_debug {
            let dbg = format!("DEBUG HALT: {} #{} (lomem={} files={} )\n", msg, n, self.lomem, self.files.len());
            msg_str = format!("{}{}", dbg, msg_str);
            // Also emit to stderr so interactive runs will show it immediately.
            eprintln!("{}", dbg.trim_end());
        }

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
            if pc as usize >= self.m.len() {
                self.halt("BAD PC", pc as i16);
            }
            let w: u16 = self.m[pc as usize] as u16;
            pc = pc.wrapping_add(1);

            if self.co_debug {
                // preview d without advancing pc
                if w & (FD_BIT as u16) != 0 {
                    let val = self.m[pc as usize];
                    eprintln!("STEP pc={} w={} preview_d={} a={} sp={}", pc.wrapping_sub(1), w, val, a, sp);
                } else {
                    let val = w >> FN_BITS;
                    eprintln!("STEP pc={} w={} preview_d={} a={} sp={}", pc.wrapping_sub(1), w, val, a, sp);
                }
            }

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

            // Update last-instruction context for write-site reporting and
            // perform immediate canary verification on watched regions (fast; small snapshots).
            self.last_pc = pc;
            self.last_sp = sp;
            self.last_a = a;
            self.instr_count = self.instr_count.wrapping_add(1);

            // Record instruction into history buffer for focused debugging
            let rec = InstrRec { pc: pc.wrapping_sub(1), w, d, a, sp };
            self.instr_history.push_back(rec);
            if self.instr_history.len() > self.history_size {
                self.instr_history.pop_front();
            }

            // Immediate per-instruction check: if any watched region (that hasn't
            // been triggered by a recorded write) changes compared to its initial
            // snapshot, that's a corruption event we want to capture immediately.
            if self.co_debug && !self.watch_regions.is_empty() {
                for wr in &self.watch_regions {
                    if wr.triggered { continue; }
                    for (offset, &orig) in wr.snapshot.iter().enumerate() {
                        let idx = wr.start + offset;
                        if idx < self.m.len() && self.m[idx] != orig {
                            eprintln!("WATCH-CORRUPT {} idx={} pc={} sp={} a={} orig={} now={}", wr.label, idx, pc, sp, a, orig, self.m[idx]);
                            let start = if idx > 40 { idx - 40 } else { 0 };
                            let end = (idx + 40).min(self.m.len());
                            eprintln!("MEM around corrupt {}..{} = {:?}", start, end, &self.m[start..end]);
                            eprintln!("watch snapshot {:?}", wr.snapshot);
                            self.dump_free_state("watch-corrupt");
                            self.scan_possible_coroutines(start, end);
                            self.dump_recent_writes(start, end);
                            self.dump_instr_history();
                            self.dump_decoded_history(80);
                            self.halt("WATCH-CORRUPT", 0);
                        }
                    }
                }
            }

            // Periodic deeper verification (less frequent) for broader snapshots
            if self.co_debug && self.instr_count % 500 == 0 {
                // verify watches for unexpected changes in snapshoted regions
                for wr in &self.watch_regions {
                    if wr.triggered { continue; }
                    // compare snapshot
                    for (offset, &orig) in wr.snapshot.iter().enumerate() {
                        let idx = wr.start + offset;
                        if idx < self.m.len() && self.m[idx] != orig {
                            eprintln!("WATCH-CANARY {} mismatch idx={} orig={} now={}", wr.label, idx, orig, self.m[idx]);
                            let start = if wr.start > 20 { wr.start - 20 } else { 0 };
                            let end = (wr.end + 20).min(self.m.len());
                            eprintln!("MEM around canary {}..{} = {:?}", start, end, &self.m[start..end]);
                        }
                    }
                }
            }
            match w & F7_X as u16 {
                0 => { // F0_L
                    b = a;
                    a = d as i16;
                }
                1 => { // F1_S
                    let d_idx = d as usize;
                    if d_idx >= self.m.len() {
                        self.halt("BAD STORE", d as i16);
                    }
                    let old_store = self.m[d_idx];
                    self.m[d_idx] = a;
                    // Instrument writes to detect the first write into watched regions
                    self.check_write_index(d_idx, pc, sp, a);
                    self.record_write(d_idx, pc, sp, a, old_store, self.m[d_idx]);
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
                        let mut v_ptr = (d_addr + 2) as usize;

                        // Diagnostic/fallback: when the computed v_ptr looks invalid
                        // (out-of-bounds or contains zeroes), scan a small neighborhood
                        // for a plausible coroutine control block and use it if found.
                        // This is gated by `co_debug` to collect provenance before
                        // applying any corrective behavior.
                        if self.co_debug {
                            let mut need_scan = false;
                            if v_ptr >= self.m.len() {
                                need_scan = true;
                                eprintln!("K-syscall: v_ptr {} out-of-bounds (len={}), scanning nearby", v_ptr, self.m.len());
                            } else if self.m[v_ptr] == 0 {
                                need_scan = true;
                                eprintln!("K-syscall: v_ptr {} looks zeroed, scanning nearby", v_ptr);
                            }
                            if need_scan {
                                let scan_start = v_ptr.saturating_sub(8);
                                let scan_end = (v_ptr + 8).min(self.m.len().saturating_sub(1));
                                for cand in scan_start..=scan_end {
                                    if self.is_plausible_coroutine(cand) {
                                        eprintln!("K-syscall: fallback found plausible coroutine at {} (was {})", cand, v_ptr);
                                        v_ptr = cand;
                                        break;
                                    }
                                }
                            }
                        }
                        match a {
                            K01_START => {}
                            K03_ABORT => { /* abort / no-op in interpreter */ }
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
                                let idx0 = b_addr as usize;
                                let old0 = self.m[idx0];
                                self.m[idx0] = sp as i16;
                                self.check_write_index(idx0, pc, sp, a);
                                self.record_write(idx0, pc, sp, a, old0, self.m[idx0]);

                                let idx1 = b_addr as usize + 1;
                                let old1 = self.m[idx1];
                                self.m[idx1] = pc as i16;
                                self.check_write_index(idx1, pc, sp, a);
                                self.record_write(idx1, pc, sp, a, old1, self.m[idx1]);

                                let idx2 = b_addr as usize + 2;
                                let old2 = self.m[idx2];
                                self.m[idx2] = d_addr as i16;  // BUG FIX: was 'd', should be 'd_addr'
                                self.check_write_index(idx2, pc, sp, a);
                                self.record_write(idx2, pc, sp, a, old2, self.m[idx2]);

                                let idx3 = b_addr as usize + 3;
                                let old3 = self.m[idx3];
                                self.m[idx3] = self.m[v_ptr + 1];
                                self.check_write_index(idx3, pc, sp, a);
                                self.record_write(idx3, pc, sp, a, old3, self.m[idx3]);

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

                                if cptr == 0 || cptr + 1 >= self.m.len() {
                                    self.halt("BAD CHANGECO C", 0);
                                }
                                if currco_addr >= self.m.len() {
                                    self.halt("BAD CURRCO", 0);
                                }

                                let currco = self.m[currco_addr] as u16 as usize;
                                if self.co_debug {
                                    eprintln!(
                                        "CHANGECO enter: arg={} currco_addr={} currco={} -> cptr={} sp={} pc={}",
                                        arg, currco_addr, currco, cptr, sp, pc
                                    );
                                    // Dump a small window of the coroutine control block at cptr
                                    if cptr < self.m.len() {
                                        let start = cptr;
                                        let end = (start + 12).min(self.m.len());
                                        eprintln!("C@{}..{} = {:?}", start, end, &self.m[start..end]);
                                    }
                                    // If we have a current coroutine vector pointer, dump it as well
                                    if currco != 0 && currco < self.m.len() {
                                        let start = currco;
                                        let end = (start + 8).min(self.m.len());
                                        eprintln!("CURRCO@{}..{} = {:?}", start, end, &self.m[start..end]);
                                    }
                                    // Also dump a small window around sp and pc for context
                                    if (sp as usize) < self.m.len() {
                                        let s = sp as usize;
                                        let start = s.saturating_sub(10);
                                        let end = (s + 10).min(self.m.len());
                                        eprintln!("MEM @sp {}..{} = {:?}", start, end, &self.m[start..end]);
                                    }
                                    if (pc as usize) < self.m.len() {
                                        let start = pc as usize;
                                        let end = (start + 12).min(self.m.len());
                                        eprintln!("MEM @pc {}..{} = {:?}", start, end, &self.m[start..end]);
                                    }
                                }
                                if currco != 0 {
                                    self.m[currco] = sp as i16;
                                    self.check_write_index(currco, pc, sp, a);
                                    self.m[currco + 1] = pc as i16;
                                    self.check_write_index(currco + 1, pc, sp, a);
                                }

                                self.m[currco_addr] = cptr as i16;
                                self.check_write_index(currco_addr, pc, sp, a);

                                // Stash the incoming argument into the coroutine frame so
                                // the coroutine entry code can find it even if register 'a'
                                // is clobbered by the entry instruction stream.
                                //
                                // Contract: interpreter writes the starter-arg into slot
                                // `C!7` (i.e., `cptr + 7`) before switching. The BCPL
                                // coroutine entry (`COROENTRY`) will check `C!7`, use the
                                // value if non-zero, and clear the slot (C!7 := 0) so it
                                // does not persist between invocations. CREATECO should
                                // initialize `C!7` to 0 when the coroutine is created.
                                if cptr + 7 < self.m.len() {
                                    if self.co_debug {
                                        let prev = self.m[cptr + 7];
                                        eprintln!("CHANGECO: stashing starter-arg at C!7 (cptr={}): prev={} -> arg={}", cptr, prev, arg);
                                    }
                                    self.m[cptr + 7] = arg;
                                    self.check_write_index(cptr + 7, pc, sp, a);
                                }

                                sp = self.m[cptr] as u16;
                                pc = self.m[cptr + 1] as u16;
                                if sp as usize >= self.m.len() || (sp as usize) < PROGSTART {
                                    self.halt("BAD CHANGECO SP", sp as i16);
                                }
                                if pc as usize >= self.m.len() || (pc as usize) < PROGSTART {
                                    self.halt("BAD CHANGECO PC", pc as i16);
                                }

                                // Also set register a to the arg for immediate visibility
                                // (back-compat with calling convention)
                                a = arg;
                                if self.co_debug {
                                    eprintln!(
                                        "CHANGECO exit: currco_addr={} currco={} sp={} pc={}",
                                        currco_addr, cptr, sp, pc
                                    );
                                    if (pc as usize) < self.m.len() {
                                        let start = pc as usize;
                                        let end = (start + 10).min(self.m.len());
                                        eprintln!("MEM @pc {}..{} = {:?}", start, end, &self.m[start..end]);
                                    }
                                }
                            }
                            _ => {
                                if self.co_debug {
                                    eprintln!(
                                        "UNKNOWN CALL: a={} v_ptr={} d_addr={} d={} sp={} pc={}",
                                        a, v_ptr, d_addr, d, sp, pc
                                    );
                                    if (pc as usize) < self.m.len() {
                                        let end = (pc as usize + 10).min(self.m.len());
                                        eprintln!("MEM @pc {}..{} = {:?}", pc, pc+10, &self.m[pc as usize..end]);
                                    }
                                    if (sp as usize) < self.m.len() {
                                        let s = sp as usize;
                                        let start = s.saturating_sub(10);
                                        let end = (s + 10).min(self.m.len());
                                        eprintln!("MEM @sp {}..{} = {:?}", start, end, &self.m[start..end]);
                                    }
                                    // If available, dump memory around the frame base 'd' we used
                                    let d_usize = d as usize;
                                    if d_usize < self.m.len() {
                                        let start = d_usize.saturating_sub(10);
                                        let end = (d_usize + 20).min(self.m.len());
                                        eprintln!("MEM @d {}..{} = {:?}", start, end, &self.m[start..end]);
                                    }

                                    // Show the vector at v_ptr (if valid) and the low-area CURRCO/COLIST words
                                    if (v_ptr as usize) < self.m.len() {
                                        let start = v_ptr as usize;
                                        let end = (start + 8).min(self.m.len());
                                        eprintln!("VEC @{}..{} = {:?}", start, end, &self.m[start..end]);
                                    }
                                    // Print assumed globals at 500/501 (CURRCO/COLIST) for context
                                    if 500 + 1 < self.m.len() {
                                        eprintln!("GLOBAL CURRCO/COLIST @500..501 = {:?}", &self.m[500..502]);
                                    }

                                    // Dump allocator state and scan for coroutine-like blocks to help
                                    // identify whether an allocation or free might have left a
                                    // stale value in the control block region.
                                    self.dump_free_state("unknown-call");
                                    let scan_start = if d_usize > 200 { d_usize - 200 } else { 0 };
                                    let scan_end = (d_usize + 200).min(self.m.len());
                                    self.scan_possible_coroutines(scan_start, scan_end);

                                    // Sanity-check the vector we printed — if it doesn't look
                                    // like a coroutine control block, emit a specific
                                    // diagnostic to aid debugging rather than a generic
                                    // UNKNOWN CALL.
                                    if (v_ptr as usize) < self.m.len() && !self.is_plausible_coroutine(v_ptr as usize) {
                                        eprintln!("BAD VEC DETECTED at {}: not plausible coroutine (v_ptr={})", v_ptr, v_ptr);
                                        // Dump extra context
                                        let start = if v_ptr as usize > 40 { v_ptr as usize - 40 } else { 0 };
                                        let end = (v_ptr as usize + 40).min(self.m.len());
                                        eprintln!("MEM around bad vec {}..{} = {:?}", start, end, &self.m[start..end]);
                                        self.dump_free_state("bad-vec");
                                        self.scan_possible_coroutines(start, end);
                                        self.dump_recent_writes(start, end);
                                        self.dump_instr_history();
                                        self.dump_decoded_history(160);
                                        self.halt("BAD VEC", a);
                                    }
                                }
                                self.halt("UNKNOWN CALL", a);
                            },
                        }
                    } else {
                        let d_idx = d_addr as usize;
                        if d_idx + 1 >= self.m.len() {
                            if self.co_debug {
                                eprintln!("BAD FRAME detected: d_addr={} d_idx={} len={} a={} sp={} pc={}", d_addr, d_idx, self.m.len(), a, sp, pc);
                                let start = if d_idx > 20 { d_idx - 20 } else { 0 };
                                let end = (d_idx + 20).min(self.m.len());
                                eprintln!("MEM around d_addr {}..{} = {:?}", start, end, &self.m[start..end]);

                                // Quick scan of a nearby region to see if nearby control blocks look sane
                                let scan_start = if d_idx > 200 { d_idx - 200 } else { 0 };
                                let scan_end = (d_idx + 200).min(self.m.len());
                                let snippet_end = (scan_start + 50).min(scan_end);
                                eprintln!("MEM scan {}..{} (snippet) = {:?}", scan_start, scan_end, &self.m[scan_start..snippet_end]);

                                // Dump allocator state for additional context
                                self.dump_free_state("bad-frame");

                                // If possible, attempt to print any coroutine-like vectors near the scan area
                                for off in (scan_start..scan_end).step_by(7) {
                                    if off + 6 < self.m.len() {
                                        let block = &self.m[off..(off + 7).min(self.m.len())];
                                        eprintln!("Possible block @{}: {:?}", off, block);
                                    }
                                }

                                // Take a more thorough scan for likely coroutine control blocks
                                self.scan_possible_coroutines(scan_start, scan_end);
                            }
                            self.halt("BAD FRAME", d_addr as i16);
                        }
                        self.m[d_idx] = sp as i16;
                        self.m[d_idx + 1] = pc as i16;
                        sp = d_addr;
                        pc = a as u16;
                    }
                }
                7 => { // F7_X
                    match d {
                        1 => {
                            let a_idx = a as u16 as usize;
                            if a_idx >= self.m.len() {
                                self.halt("BAD LOAD", a);
                            }
                            a = self.m[a_idx];
                        }
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
