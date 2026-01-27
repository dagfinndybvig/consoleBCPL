const fs = require('fs');
const path = require('path');

// Constants
const ASC_TAB = 8;
const ASC_LF = 10;
const ASC_FF = 12;
const ASC_CR = 13;
const ASC_SPACE = 32;
const ASC_DOLLAR = 36;
const ASC_PERCENT = 37;
const ASC_PLUS = 43;
const ASC_MINUS = 45;
const ASC_SLASH = 47;

const ASC_0 = 48;
const ASC_9 = 57;
const ASC_A = 65;
const ASC_Z = 90;

const ASC_L = 76;
const ASC_S = 83;
const ASC_J = 74;
const ASC_T = 84;
const ASC_F = 70;
const ASC_K = 75;
const ASC_X = 88;
const ASC_C = 67;
const ASC_D = 68;
const ASC_G = 71;
const ASC_I = 73;
const ASC_P = 80;
const ASC_O = 79;
const ASC_N = 78;

const STR_NO_INPUT = "NO INPUT";
const STR_NO_OUTPUT = "NO OUTPUT";
const STR_NO_ICFILE = "NO ICFILE";
const STR_INVALID_OPTION = "INVALID OPTION";
const STR_DUPLICATE_LABEL = "DUPLICATE LABEL";
const STR_BAD_CODE_AT_P = "BAD CODE AT P";
const STR_UNSET_LABEL = "UNSET LABEL";
const STR_BAD_CH = "BAD CH";
const STR_UNKNOWN_CALL = "UNKNOWN CALL";
const STR_UNKNOWN_EXEC = "UNKNOWN EXEC";
const STR_INTCODE_ERROR_AT_PC = "INTCODE ERROR AT PC";
const STR_USAGE = "USAGE: node icint.js ICFILE [...] [-iINPUT] [-oOUTPUT]";

const PROGSTART = 401;
const WORDCOUNT = 19900;
const LABVCOUNT = 500;

const FN_BITS = 8;
const FN_MASK = 255;
const F0_L = 0;
const F1_S = 1;
const F2_A = 2;
const F3_J = 3;
const F4_T = 4;
const F5_F = 5;
const F6_K = 6;
const F7_X = 7;
const FI_BIT = 1 << 3;
const FP_BIT = 1 << 4;
const FD_BIT = 1 << 5;

// K-codes
const K01_START = 1;
const K02_SETPM = 2;
const K03_ABORT = 3;
const K04_BACKTRACE = 4;
const K11_SELECTINPUT = 11;
const K12_SELECTOUTPUT = 12;
const K13_RDCH = 13;
const K14_WRCH = 14;
const K15_UNRDCH = 15;
const K16_INPUT = 16;
const K17_OUTPUT = 17;
const K30_STOP = 30;
const K31_LEVEL = 31;
const K32_LONGJUMP = 32;
const K34_BINWRCH = 34;
const K35_REWIND = 35;
const K40_APTOVEC = 40;
const K41_FINDOUTPUT = 41;
const K42_FINDINPUT = 42;
const K46_ENDREAD = 46;
const K47_ENDWRITE = 47;
const K60_WRITES = 60;
const K62_WRITEN = 62;
const K63_NEWLINE = 63;
const K64_NEWPAGE = 64;
const K65_WRITEO = 65;
const K66_PACKSTRING = 66;
const K67_UNPACKSTRING = 67;
const K68_WRITED = 68;
const K69_WRITEARG = 69;
const K70_READN = 70;
const K71_TERMINATOR = 71;
const K74_WRITEX = 74;
const K75_WRITEHEX = 75;
const K76_WRITEF = 76;
const K77_WRITEOCT = 77;
const K78_MAPSTORE = 78;
const K85_GETBYTE = 85;
const K86_PUTBYTE = 86;
const K87_GETVEC = 87;
const K88_FREEVEC = 88;
const K89_RANDOM = 89;
const K90_CHANGECO = 90;
const K91_RESULT2 = 91;

const ENDSTREAMCH = -1;
const BYTESPERWORD = 2;

// Memory
const buffer = new ArrayBuffer(WORDCOUNT * BYTESPERWORD);
const m = new Int16Array(buffer);
const mu = new Uint16Array(buffer); // Unsigned view

let lomem = 0;
let himem = WORDCOUNT - 1;
let cis = 0;
let coutfd = 0;
let sysin = 0;
let sysprint = 0;
let vecfree = 0; // free-list head for vector allocator

// Helper functions
function cstr(s_ptr) {
    const memBytes = new Uint8Array(buffer);
    let byteIdx = s_ptr * 2;
    let len = memBytes[byteIdx];
    let str = "";
    for (let i = 0; i < len; i++) {
        str += String.fromCharCode(memBytes[byteIdx + 1 + i]);
    }
    return str;
}

function bstr(s) {
    return s;
}

function decval(c) {
    if (c >= ASC_0 && c <= ASC_9) return c - ASC_0;
    if (c >= ASC_A && c <= ASC_Z) return c - ASC_A + 10;
    return 0;
}

const strdigits = "0123456789ABCDEF";

function openfile(fn, mode) {
    if (fn.toUpperCase() === "SYSIN") return sysin;
    if (fn.toUpperCase() === "SYSPRINT") return sysprint;
    try {
        let flags = 'r';
        if (mode === 'w') flags = 'w';
        try {
            const fd = fs.openSync(fn, flags);
            return fd + 1; // 1-based
        } catch (e) {
            if (flags === 'r' && fn !== fn.toLowerCase()) {
                try {
                    const fd = fs.openSync(fn.toLowerCase(), flags);
                    return fd + 1;
                } catch (e2) {
                    return 0;
                }
            }
            return 0;
        }
    } catch (e) {
        return 0;
    }
}

function findinput(fn_bcpl) {
    let fn;
    if (typeof fn_bcpl === 'number') {
        fn = cstr(fn_bcpl);
    } else {
        fn = fn_bcpl;
    }
    return openfile(fn, 'r');
}

function findoutput(fn_bcpl) {
    let fn;
    if (typeof fn_bcpl === 'number') {
        fn = cstr(fn_bcpl);
    } else {
        fn = fn_bcpl;
    }
    return openfile(fn, 'w');
}

function endread() {
    if (cis !== sysin) {
        fs.closeSync(cis - 1);
        cis = sysin;
    }
}

function endwrite() {
    if (coutfd !== sysprint) {
        fs.closeSync(coutfd - 1);
        coutfd = sysprint;
    }
}

function rdch() {
    const buffer1 = Buffer.alloc(1);
    try {
        const bytesRead = fs.readSync(cis - 1, buffer1, 0, 1, null);
        if (bytesRead !== 1) return ENDSTREAMCH;
        let c = buffer1[0];
        return c === ASC_CR ? ASC_LF : c;
    } catch (e) {
        return ENDSTREAMCH;
    }
}

function wrch(c) {
    if (c === ASC_LF) {
        newline();
    } else {
        const buffer1 = Buffer.from([c]);
        fs.writeSync(coutfd - 1, buffer1, 0, 1);
    }
}

function newline() {
    fs.writeSync(coutfd - 1, "\n");
}

function writes(s_ptr) {
    const memBytes = new Uint8Array(buffer);
    let byteIdx = s_ptr * 2;
    let len = memBytes[byteIdx];
    for (let i = 0; i < len; i++) {
        wrch(memBytes[byteIdx + 1 + i]);
    }
}

function writed(n, d) {
    let s = Math.abs(n).toString();
    if (n < 0) s = "-" + s;
    while (s.length < d) s = " " + s;
    for (let i = 0; i < s.length; i++) {
        wrch(s.charCodeAt(i));
    }
}

function writen(n) {
    writed(n, 0);
}

function readn() {
    let sum = 0;
    let c;
    let neg = false;
    do {
        c = rdch();
    } while (c === ASC_SPACE || c === ASC_LF || c === ASC_TAB);
    if (c === ASC_MINUS) {
        neg = true;
        c = rdch();
    } else if (c === ASC_PLUS) {
        c = rdch();
    }
    while (c >= ASC_0 && c <= ASC_9) {
        sum = sum * 10 + (c - ASC_0);
        c = rdch();
    }
    m[K71_TERMINATOR] = c;
    return neg ? -sum : sum;
}

function writeoct(n, d) {
    if (d > 1) writeoct(n >>> 3, d - 1);
    wrch(strdigits.charCodeAt(n & 7));
}

function writehex(n, d) {
    if (d > 1) writehex(n >>> 4, d - 1);
    wrch(strdigits.charCodeAt(n & 15));
}

function writef(v_ptr) {
    let fmt_ptr = m[v_ptr++];
    const memBytes = new Uint8Array(buffer);
    let byteIdx = fmt_ptr * 2;
    let len = memBytes[byteIdx];
    let ss = 1;
    while (ss <= len) {
        let c = memBytes[byteIdx + ss++];
        if (c !== ASC_PERCENT) {
            wrch(c);
        } else {
            c = memBytes[byteIdx + ss++];
            switch (c) {
                default: wrch(c); break;
                case ASC_S: writes(m[v_ptr++]); break;
                case ASC_C: wrch(m[v_ptr++]); break;
                case ASC_O: writeoct(mu[v_ptr++], decval(memBytes[byteIdx + ss++])); break;
                case ASC_X: writehex(mu[v_ptr++], decval(memBytes[byteIdx + ss++])); break;
                case ASC_I: writed(m[v_ptr++], decval(memBytes[byteIdx + ss++])); break;
                case ASC_N: writen(m[v_ptr++]); break;
            }
        }
    }
}

function packstring(v_ptr, s_ptr) {
    let len = m[v_ptr];
    let n = Math.floor(len / BYTESPERWORD);
    m[s_ptr + n] = 0;
    const memBytes = new Uint8Array(buffer);
    let byteDest = s_ptr * 2;
    for (let i = 0; i <= len; i++) {
        memBytes[byteDest + i] = m[v_ptr + i] & 0xFF;
    }
    return n;
}

function unpackstring(s_ptr, v_ptr) {
    const memBytes = new Uint8Array(buffer);
    let byteSrc = s_ptr * 2;
    let len = memBytes[byteSrc];
    for (let i = 0; i <= len; i++) {
        m[v_ptr + i] = memBytes[byteSrc + i];
    }
}

// Vector allocator (matches the C implementation):
// Block header at h: m[h] = size (words), m[h+1] = next free header
// Payload starts at h+2; GETVEC returns pointer to payload (h+2)
function allocvec(n) {
    if (n <= 0) return 0;
    let prev = 0;
    let cur = vecfree;
    while (cur) {
        let sz = m[cur];
        let next = m[cur + 1];
        if (sz >= n) {
            if (prev) m[prev + 1] = next; else vecfree = next;
            return cur + 2;
        }
        prev = cur;
        cur = next;
    }
    let total = n + 2;
    let h = himem - total + 1;
    if (h <= lomem) return 0;
    m[h] = n; m[h + 1] = 0;
    himem = h - 1;
    return h + 2;
}

function freevec(p) {
    if (!p) return;
    let h = p - 2;
    m[h + 1] = vecfree;
    vecfree = h;
}

// Assembler variables
let cp = 0;
let ch = 0;
const labv_offset = WORDCOUNT - LABVCOUNT;

function stw(w) {
    m[lomem++] = w;
    cp = 0;
}

function stc(c) {
    if (cp === 0) stw(0);
    const memBytes = new Uint8Array(buffer);
    let byteAddr = (lomem - 1) * 2 + cp;
    memBytes[byteAddr] = c;
    cp++;
    if (cp === BYTESPERWORD) cp = 0;
}

function rch() {
    ch = rdch();
    while (ch === ASC_SLASH) {
        do { ch = rdch(); } while (ch !== ASC_LF && ch !== ENDSTREAMCH);
        while (ch === ASC_LF) ch = rdch();
    }
}

function rdn() {
    let sum = 0;
    let neg = (ch === ASC_MINUS);
    if (neg) rch();
    while (ch >= ASC_0 && ch <= ASC_9) {
        sum = sum * 10 + (ch - ASC_0);
        rch();
    }
    return neg ? -sum : sum;
}

function labref(n, a) {
    let k = m[labv_offset + n];
    if (k < 0) k = -k; else m[labv_offset + n] = a;
    m[a] += k;
}

function halt(msg, n) {
    coutfd = sysprint;
    const str = msg + (n ? " #" + n : "") + "\n";
    fs.writeSync(coutfd - 1, str);
    process.exit(-1);
}

function assemble() {
    let n;
    for (let i = 0; i < LABVCOUNT; i++) m[labv_offset + i] = 0;
    cp = 0;
    rch();
    while (true) {
        if (ch <= ASC_9 && ch >= ASC_0) {
            n = rdn();
            let k = m[labv_offset + n];
            if (k < 0) halt(STR_DUPLICATE_LABEL, n);
            while (k > 0) {
                let tmp = m[k];
                m[k] = lomem;
                k = tmp;
            }
            m[labv_offset + n] = -lomem;
            cp = 0;
            continue;
        }
        switch (ch) {
            default:
                if (ch !== ENDSTREAMCH) halt(STR_BAD_CH, ch);
                return;
            case ASC_DOLLAR:
            case ASC_SPACE:
            case ASC_LF:
                rch();
                continue;
            case ASC_L: n = F0_L; break;
            case ASC_S: n = F1_S; break;
            case ASC_A: n = F2_A; break;
            case ASC_J: n = F3_J; break;
            case ASC_T: n = F4_T; break;
            case ASC_F: n = F5_F; break;
            case ASC_K: n = F6_K; break;
            case ASC_X: n = F7_X; break;
            case ASC_C:
                rch(); stc(rdn()); continue;
            case ASC_D:
                rch();
                if (ch === ASC_L) {
                    rch(); stw(0); labref(rdn(), lomem - 1);
                } else {
                    stw(rdn());
                }
                continue;
            case ASC_G:
                rch(); n = rdn();
                if (ch === ASC_L) rch(); else halt(STR_BAD_CODE_AT_P, lomem);
                m[n] = 0; labref(rdn(), n);
                continue;
            case ASC_Z:
                for (n = 0; n < LABVCOUNT; ++n) {
                    if (m[labv_offset + n] > 0) halt(STR_UNSET_LABEL, n);
                }
                for (let i = 0; i < LABVCOUNT; i++) m[labv_offset + i] = 0;
                cp = 0;
                rch();
                continue;
        }
        rch();
        if (ch === ASC_I) { n |= FI_BIT; rch(); }
        if (ch === ASC_P) { n |= FP_BIT; rch(); }
        if (ch === ASC_G) { rch(); }
        if (ch === ASC_L) {
            rch();
            stw(n | FD_BIT);
            stw(0);
            labref(rdn(), lomem - 1);
        } else {
            let d = rdn();
            if ((d & FN_MASK) === d) {
                stw(n | (d << FN_BITS));
            } else {
                stw(n | FD_BIT);
                stw(d);
            }
        }
    }
}

let trace_mode = false;

function interpret() {
    let pc = PROGSTART;
    let sp = lomem;
    let a = 0;
    let b = 0;
    let w, d;
    let v_ptr;
    while (true) {
        w = mu[pc++];
        if (w & FD_BIT) {
            d = m[pc++];
        } else {
            d = w >>> FN_BITS;
        }
        if (w & FP_BIT) d += sp;
        if (w & FI_BIT) d = m[d];
        switch (w & F7_X) {
            case F0_L: b = a; a = d; break;
            case F1_S: m[d] = a; break;
            case F2_A: a = (a + d) << 16 >> 16; break;
            case F3_J: pc = d; break;
            case F4_T: if (a !== 0) pc = d; break;
            case F5_F: if (a === 0) pc = d; break;
            case F6_K:
                d += sp;
                if (a < PROGSTART) {
                    v_ptr = d + 2;
                    switch (a) {
                        default: halt(STR_UNKNOWN_CALL, a);
                        case K01_START: break;
                        case K02_SETPM: 
                            m[sp] = 0; // Previous SP
                            m[sp + 1] = PROGSTART + 2; // Return PC
                            pc = a;
                            break;
                        case K03_ABORT: break;
                        case K04_BACKTRACE: break;
                        case K11_SELECTINPUT: cis = m[v_ptr]; break;
                        case K12_SELECTOUTPUT: coutfd = m[v_ptr]; break;
                        case K13_RDCH: a = rdch(); break;
                        case K14_WRCH: wrch(m[v_ptr]); break;
                        case K16_INPUT: a = cis; break;
                        case K17_OUTPUT: a = coutfd; break;
                        case K30_STOP: return m[v_ptr];
                        case K31_LEVEL: a = sp; break;
                        case K32_LONGJUMP: sp = m[v_ptr]; pc = m[v_ptr + 1]; break;
                        case K40_APTOVEC:
                            b = d + m[v_ptr + 1] + 1;
                            m[b] = sp; m[b + 1] = pc; m[b + 2] = d; m[b + 3] = m[v_ptr + 1];
                            sp = b; pc = m[v_ptr];
                            break;
                        case K41_FINDOUTPUT: a = findoutput(m[v_ptr]); break;
                        case K42_FINDINPUT: a = findinput(m[v_ptr]); break;
                        case K46_ENDREAD: endread(); break;
                        case K47_ENDWRITE: endwrite(); break;
                        case K60_WRITES: writes(m[v_ptr]); break;
                        case K62_WRITEN: writen(m[v_ptr]); break;
                        case K63_NEWLINE: newline(); break;
                        case K64_NEWPAGE: wrch(ASC_FF); break;
                        case K66_PACKSTRING: a = packstring(m[v_ptr], m[v_ptr + 1]); break;
                        case K67_UNPACKSTRING: unpackstring(m[v_ptr], m[v_ptr + 1]); break;
                        case K68_WRITED: writed(m[v_ptr], m[v_ptr + 1]); break;
                        case K70_READN: a = readn(); break;
                        case K75_WRITEHEX: writehex(mu[v_ptr], m[v_ptr + 1]); break;
                        case K77_WRITEOCT: writeoct(mu[v_ptr], m[v_ptr + 1]); break;
                        case K76_WRITEF: writef(v_ptr); break;
                        case K85_GETBYTE: {
                            const memBytes = new Uint8Array(buffer);
                            let base = m[v_ptr] * 2;
                            let offset = m[v_ptr + 1];
                            a = memBytes[base + offset];
                        } break;
                        case K86_PUTBYTE: {
                            const memBytes = new Uint8Array(buffer);
                            let base = m[v_ptr] * 2;
                            let offset = m[v_ptr + 1];
                            memBytes[base + offset] = m[v_ptr + 2];
                        } break;
                        case K87_GETVEC: a = allocvec(m[v_ptr]); break;
                        case K88_FREEVEC: freevec(m[v_ptr]); break;
                        case K90_CHANGECO: {
                            let arg = m[v_ptr];
                            let cptr = m[v_ptr + 1];
                            let currco_addr = m[v_ptr + 2];
                            let currco;

                            if (cptr <= 0 || cptr + 6 >= WORDCOUNT) halt("BAD CHANGECO C", 0);
                            if (currco_addr < 0 || currco_addr >= WORDCOUNT) halt("BAD CURRCO", 0);

                            currco = m[currco_addr];
                            if (currco !== 0) {
                                if (currco < 0 || currco + 1 >= WORDCOUNT) halt("BAD CURRCO VAL", 0);
                                m[currco] = sp;
                                m[currco + 1] = pc;
                            }

                            m[currco_addr] = cptr;
                            sp = m[cptr];
                            pc = m[cptr + 1];
                            if ((sp >= WORDCOUNT) || (sp < PROGSTART)) halt("BAD CHANGECO SP", sp);
                            if ((pc >= WORDCOUNT) || (pc < PROGSTART)) halt("BAD CHANGECO PC", pc);
                            a = arg;
                        } break;
                    }
                } else {
                    m[d] = sp; m[d + 1] = pc; sp = d; pc = a;
                }
                break;
            case F7_X:
                switch (d) {
                    default: halt(STR_UNKNOWN_EXEC, d);
                    case 1: a = m[a]; break;
                    case 2: a = (-a) << 16 >> 16; break;
                    case 3: a = (~a) << 16 >> 16; break;
                    case 4: pc = m[sp + 1]; sp = m[sp]; break;
                    case 5: a = Math.imul(b, a) << 16 >> 16; break;
                    case 6: if (a !== 0) a = Math.trunc(b / a) << 16 >> 16; break;
                    case 7: if (a !== 0) a = (b % a) << 16 >> 16; break;
                    case 8: a = (b + a) << 16 >> 16; break;
                    case 9: a = (b - a) << 16 >> 16; break;
                    case 10: a = -(b === a); break;
                    case 11: a = -(b !== a); break;
                    case 12: a = -(b < a); break;
                    case 13: a = -(b >= a); break;
                    case 14: a = -(b > a); break;
                    case 15: a = -(b <= a); break;
                    case 16: a = (b << a) << 16 >> 16; break;
                    case 17: a = ((b & 0xFFFF) >>> a) << 16 >> 16; break;
                    case 18: a = (b & a) << 16 >> 16; break;
                    case 19: a = (b | a) << 16 >> 16; break;
                    case 20: a = (b ^ a) << 16 >> 16; break;
                    case 21: a = (b ^ ~a) << 16 >> 16; break;
                    case 22: return 0;
                    case 23: {
                        let v_idx = pc;
                        b = m[v_idx++];
                        pc = m[v_idx++]; 
                        let found = false;
                        while (b--) {
                            if (a === m[v_idx]) {
                                pc = m[v_idx + 1];
                                found = true;
                                break;
                            }
                            v_idx += 2;
                        }
                    } break;
                }
                break;
        }
    }
}

function loadcode(fn) {
    const f = findinput(fn);
    if (f) {
        cis = f;
        assemble();
        endread();
    }
    return f;
}

function init() {
    // Reserve top-of-memory area for label table and metadata so
    // GETVEC allocations won't overlap it at runtime. Initialize
    // the vector free-list.
    himem = WORDCOUNT - LABVCOUNT - 1;
    vecfree = 0;
    for (lomem = 0; lomem < PROGSTART; ++lomem) m[lomem] = lomem;
    stw(F0_L | FI_BIT | (K01_START << FN_BITS));
    stw(F6_K | (2 << FN_BITS));
    stw(F7_X | (22 << FN_BITS));
    cis = sysin = 1;
    coutfd = sysprint = 2;
}

function pipeinput(fn) {
    const f = openfile(fn, 'r');
    if (!f) halt(STR_NO_INPUT, 0);
    cis = sysin = f;
}

function pipeoutput(fn) {
    const f = openfile(fn, 'w');
    if (!f) halt(STR_NO_OUTPUT, 0);
    coutfd = sysprint = f;
}

function main() {
    init();
    const args = process.argv.slice(2);
    if (args.length === 0) {
        console.log(STR_USAGE);
        process.exit(0);
    }
    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg.startsWith('-')) {
            if (arg.startsWith('-i')) {
                pipeinput(arg.substring(2));
            } else if (arg.startsWith('-o')) {
                pipeoutput(arg.substring(2));
            } else if (arg === '-trace') {
                trace_mode = true;
            } else {
                halt(STR_INVALID_OPTION, i);
            }
        } else {
            if (!loadcode(arg)) halt(STR_NO_ICFILE, 0);
        }
    }
    interpret();
}

main();