#!/usr/bin/env python3
"""
BCPL INTCODE Interpreter - Coroutine-enabled Python version

This variant extends `icint.py` with coroutine support on the model of
`new_coroutines/icint.c`, specifically:
- K87 GETVEC
- K88 FREEVEC
- K90 CHANGECO

It reuses the assembler and most runtime services from `icint.py`.

Usage:
    /bin/python3 icint_co.py TEST2
    /bin/python3 icint_co.py RUNABLE

Full parity run helper:
    ./run_all_python_tests.sh
"""

import sys
import icint as base

K90_CHANGECO = 90

co_debug = True
vecfree = 0


def allocvec(n):
    """Allocate a vector of n words from free-list or top-of-memory."""
    global vecfree

    if n <= 0:
        return 0

    prev = 0
    cur = vecfree

    while cur:
        sz = base.m[cur]
        nxt = base.m[cur + 1]
        if sz >= n:
            if prev:
                base.m[prev + 1] = nxt
            else:
                vecfree = nxt
            return cur + 2
        prev = cur
        cur = nxt

    total = n + 2
    h = base.himem - total + 1
    if h <= base.lomem:
        return 0

    base.m[h] = n
    base.m[h + 1] = 0
    base.himem = h - 1
    return h + 2


def freevec(p):
    """Free a vector previously allocated by allocvec."""
    global vecfree

    if not p:
        return

    h = p - 2
    base.m[h + 1] = vecfree
    vecfree = h


def interpret():
    """Execute INTCODE starting from PROGSTART with coroutine support."""
    _m = base.m

    _PROGSTART = base.PROGSTART
    _FD_BIT = base.FD_BIT
    _FP_BIT = base.FP_BIT
    _FI_BIT = base.FI_BIT
    _FN_BITS = base.FN_BITS

    pc = _PROGSTART
    sp = base.lomem
    a = 0
    b = 0

    def _s16(val):
        val &= 0xFFFF
        return val - 0x10000 if val >= 0x8000 else val

    while True:
        w = _m[pc] & 0xFFFF
        pc += 1

        if w & _FD_BIT:
            d = _m[pc]
            pc += 1
        else:
            d = w >> _FN_BITS

        if w & _FP_BIT:
            d = _s16(d + sp)
        if w & _FI_BIT:
            d = _m[d]

        fn = w & 7

        if fn == 0:  # L
            b = a
            a = d
        elif fn == 1:  # S
            _m[d] = a
        elif fn == 2:  # A
            a = _s16(a + d)
        elif fn == 3:  # J
            pc = d
        elif fn == 4:  # T
            if a != 0:
                pc = d
        elif fn == 5:  # F
            if a == 0:
                pc = d
        elif fn == 6:  # K
            d = _s16(d + sp)

            if a < _PROGSTART:
                v_ptr = d + 2

                if a == 1:  # K01_START
                    pass
                elif a == 2:  # K02_SETPM
                    _m[sp] = 0
                    _m[sp + 1] = _PROGSTART + 2
                    pc = a
                elif a == 3 or a == 4:  # K03_ABORT, K04_BACKTRACE
                    pass
                elif a == 11:  # K11_SELECTINPUT
                    base.cis = _m[v_ptr]
                elif a == 12:  # K12_SELECTOUTPUT
                    base.cos = _m[v_ptr]
                elif a == 13:  # K13_RDCH
                    a = base.rdch()
                elif a == 14:  # K14_WRCH
                    base.wrch(_m[v_ptr])
                elif a == 16:  # K16_INPUT
                    a = base.cis
                elif a == 17:  # K17_OUTPUT
                    a = base.cos
                elif a == 30:  # K30_STOP
                    return _m[v_ptr]
                elif a == 31:  # K31_LEVEL
                    a = sp
                elif a == 32:  # K32_LONGJUMP
                    sp = _m[v_ptr]
                    pc = _m[v_ptr + 1]
                elif a == 40:  # K40_APTOVEC
                    b = d + _m[v_ptr + 1] + 1
                    _m[b] = sp
                    _m[b + 1] = pc
                    _m[b + 2] = d
                    _m[b + 3] = _m[v_ptr + 1]
                    sp = b
                    pc = _m[v_ptr]
                elif a == 41:  # K41_FINDOUTPUT
                    a = base.findoutput(_m[v_ptr])
                elif a == 42:  # K42_FINDINPUT
                    a = base.findinput(_m[v_ptr])
                elif a == 46:  # K46_ENDREAD
                    base.endread()
                elif a == 47:  # K47_ENDWRITE
                    base.endwrite()
                elif a == 60:  # K60_WRITES
                    base.writes(_m[v_ptr])
                elif a == 62:  # K62_WRITEN
                    base.writen(_m[v_ptr])
                elif a == 63:  # K63_NEWLINE
                    base.newline()
                elif a == 64:  # K64_NEWPAGE
                    base.wrch(12)
                elif a == 66:  # K66_PACKSTRING
                    a = base.packstring(_m[v_ptr], _m[v_ptr + 1])
                elif a == 67:  # K67_UNPACKSTRING
                    base.unpackstring(_m[v_ptr], _m[v_ptr + 1])
                elif a == 68:  # K68_WRITED
                    base.writed(_m[v_ptr], _m[v_ptr + 1])
                elif a == 70:  # K70_READN
                    a = base.readn()
                elif a == 75:  # K75_WRITEHEX
                    base.writehex(_m[v_ptr] & 0xFFFF, _m[v_ptr + 1])
                elif a == 76:  # K76_WRITEF
                    base.writef(v_ptr)
                elif a == 77:  # K77_WRITEOCT
                    base.writeoct(_m[v_ptr] & 0xFFFF, _m[v_ptr + 1])
                elif a == 85:  # K85_GETBYTE
                    base_addr = _m[v_ptr] * 2
                    offset = _m[v_ptr + 1]
                    a = base._get_byte(base_addr + offset)
                elif a == 86:  # K86_PUTBYTE
                    base_addr = _m[v_ptr] * 2
                    offset = _m[v_ptr + 1]
                    base._set_byte(base_addr + offset, _m[v_ptr + 2])
                elif a == 87:  # K87_GETVEC
                    a = allocvec(_m[v_ptr])
                elif a == 88:  # K88_FREEVEC
                    freevec(_m[v_ptr])
                elif a == K90_CHANGECO:  # K90_CHANGECO
                    arg = _m[v_ptr]
                    cptr = _m[v_ptr + 1]
                    currco_addr = _m[v_ptr + 2]

                    if cptr <= 0 or cptr + 6 >= base.WORDCOUNT:
                        base.halt("BAD CHANGECO C", 0)
                    if currco_addr < 0 or currco_addr >= base.WORDCOUNT:
                        base.halt("BAD CURRCO", 0)

                    currco = _m[currco_addr]

                    if co_debug:
                        sys.stderr.write(
                            f"CHANGECO enter: arg={arg} cptr={cptr} currco_addr={currco_addr} "
                            f"currco={currco} sp={sp} pc={pc}\n"
                        )
                        sys.stderr.write(
                            "  cptr fields: "
                            f"[0]sp={_m[cptr]} [1]pc={_m[cptr+1]} [2]parent={_m[cptr+2]} "
                            f"[3]next={_m[cptr+3]} [4]fn={_m[cptr+4]} [5]size={_m[cptr+5]} "
                            f"[6]self={_m[cptr+6]}\n"
                        )

                    if currco != 0:
                        if currco < 0 or currco + 1 >= base.WORDCOUNT:
                            base.halt("BAD CURRCO VAL", 0)
                        _m[currco] = sp
                        _m[currco + 1] = pc
                        if co_debug:
                            sys.stderr.write(
                                f"  saved currco[{currco}]: sp={sp} pc={pc}\n"
                            )

                    _m[currco_addr] = cptr
                    sp = _m[cptr]
                    pc = _m[cptr + 1]

                    if co_debug:
                        sys.stderr.write(
                            f"CHANGECO exit: new_sp={sp} new_pc={pc} a={arg}\n"
                        )

                    if sp >= base.WORDCOUNT or sp < base.PROGSTART:
                        base.halt("BAD CHANGECO SP", sp)
                    if pc >= base.WORDCOUNT or pc < base.PROGSTART:
                        base.halt("BAD CHANGECO PC", pc)

                    a = arg
                else:
                    base.halt(base.STR_UNKNOWN_CALL, a)
            else:
                _m[d] = sp
                _m[d + 1] = pc
                sp = d
                pc = a

        elif fn == 7:  # X
            if d == 1:
                a = _m[a]
            elif d == 2:
                a = _s16(-a)
            elif d == 3:
                a = _s16(~a)
            elif d == 4:
                pc = _m[sp + 1]
                sp = _m[sp]
            elif d == 5:
                a = _s16(b * a)
            elif d == 6:
                if a != 0:
                    sign = -1 if (b < 0) != (a < 0) else 1
                    a = sign * (abs(b) // abs(a))
            elif d == 7:
                if a != 0:
                    if b < 0:
                        a = -(abs(b) % abs(a))
                    else:
                        a = abs(b) % abs(a)
            elif d == 8:
                a = _s16(b + a)
            elif d == 9:
                a = _s16(b - a)
            elif d == 10:
                a = -1 if (b == a) else 0
            elif d == 11:
                a = -1 if (b != a) else 0
            elif d == 12:
                a = -1 if (b < a) else 0
            elif d == 13:
                a = -1 if (b >= a) else 0
            elif d == 14:
                a = -1 if (b > a) else 0
            elif d == 15:
                a = -1 if (b <= a) else 0
            elif d == 16:
                a = _s16(b << a)
            elif d == 17:
                a = _s16((b & 0xFFFF) >> a)
            elif d == 18:
                a = _s16(b & a)
            elif d == 19:
                a = _s16(b | a)
            elif d == 20:
                a = _s16(b ^ a)
            elif d == 21:
                a = _s16(b ^ ~a)
            elif d == 22:
                return 0
            elif d == 23:
                v_idx = pc
                cnt = _m[v_idx]
                v_idx += 1
                pc = _m[v_idx]
                v_idx += 1

                while cnt > 0:
                    if a == _m[v_idx]:
                        pc = _m[v_idx + 1]
                        break
                    v_idx += 2
                    cnt -= 1
            else:
                base.halt(base.STR_UNKNOWN_EXEC, d)


def init():
    """Initialize base runtime with coroutine allocator bounds."""
    global vecfree

    base.init()
    base.himem = base.WORDCOUNT - base.LABVCOUNT - 1
    vecfree = 0


def main():
    init()

    args = sys.argv[1:]
    if not args:
        print(base.STR_USAGE)
        sys.exit(0)

    for arg in args:
        if arg.startswith('-'):
            if arg.startswith('-i'):
                base.pipeinput(arg[2:])
            elif arg.startswith('-o'):
                base.pipeoutput(arg[2:])
            else:
                base.halt(base.STR_INVALID_OPTION)
        else:
            if not base.loadcode(arg):
                base.halt(base.STR_NO_ICFILE)

    result = interpret()
    sys.exit(result)


if __name__ == "__main__":
    main()
