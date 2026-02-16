package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

const (
	ascTab     = 8
	ascLF      = 10
	ascFF      = 12
	ascCR      = 13
	ascSpace   = 32
	ascDollar  = 36
	ascPercent = 37
	ascPlus    = 43
	ascMinus   = 45
	ascSlash   = 47
	asc0       = 48
	asc9       = 57
	ascA       = 65
	ascZ       = 90
	ascL       = 76
	ascS       = 83
	ascJ       = 74
	ascT       = 84
	ascF       = 70
	ascK       = 75
	ascX       = 88
	ascC       = 67
	ascD       = 68
	ascG       = 71
	ascI       = 73
	ascP       = 80
	ascO       = 79
	ascN       = 78

	progStart = 401
	wordCount = 19900
	labvCount = 500

	fnBits = 8
	fnMask = 255
	f0L    = 0
	f1S    = 1
	f2A    = 2
	f3J    = 3
	f4T    = 4
	f5F    = 5
	f6K    = 6
	f7X    = 7
	fiBit  = 1 << 3
	fpBit  = 1 << 4
	fdBit  = 1 << 5

	k11SelectInput = 11
	k12SelectOutput = 12
	k13RDCH        = 13
	k14WRCH        = 14
	k16Input       = 16
	k17Output      = 17
	k30STOP        = 30
	k31LEVEL       = 31
	k32LONGJUMP    = 32
	k40APTOVEC     = 40
	k41FindOutput  = 41
	k42FindInput   = 42
	k46EndRead     = 46
	k47EndWrite    = 47
	k60Writes      = 60
	k62Writen      = 62
	k63Newline     = 63
	k64Newpage     = 64
	k66PackString  = 66
	k67UnpackString = 67
	k68Writed      = 68
	k70Readn       = 70
	k71Terminator  = 71
	k75Writehex    = 75
	k76Writef      = 76
	k77Writeoct    = 77
	k85Getbyte     = 85
	k86Putbyte     = 86
	k87Getvec      = 87
	k88Freevec     = 88
	k90Changeco    = 90

	endStreamCh = -1
	bytesPerWord = 2
)

var (
	m          = make([]int, wordCount)
	lomem      int
	himem      = wordCount - 1
	cis        int
	cos        int
	sysin      int
	sysprint   int
	cp         int
	ch         int
	labvOffset = wordCount - labvCount
	vecfree    int
	coDebug    = true

	fileHandles = map[int]*os.File{}
	inputReaders = map[int]*bufio.Reader{}
	nextHandle   = 10
)

func s16(v int) int {
	v &= 0xFFFF
	if v >= 0x8000 {
		return v - 0x10000
	}
	return v
}

func muGet(idx int) int { return m[idx] & 0xFFFF }

func mSet(idx, v int) { m[idx] = s16(v) }

func getByte(byteIdx int) int {
	wordIdx := byteIdx >> 1
	val := m[wordIdx] & 0xFFFF
	if byteIdx&1 == 1 {
		return (val >> 8) & 0xFF
	}
	return val & 0xFF
}

func setByte(byteIdx, v int) {
	wordIdx := byteIdx >> 1
	cur := m[wordIdx] & 0xFFFF
	v &= 0xFF
	var n int
	if byteIdx&1 == 1 {
		n = (cur & 0x00FF) | (v << 8)
	} else {
		n = (cur & 0xFF00) | v
	}
	m[wordIdx] = s16(n)
}

func cstr(sPtr int) string {
	byteIdx := sPtr * 2
	length := getByte(byteIdx)
	b := make([]byte, 0, length)
	for i := 0; i < length; i++ {
		b = append(b, byte(getByte(byteIdx+1+i)))
	}
	return string(b)
}

func decval(c int) int {
	if c >= asc0 && c <= asc9 {
		return c - asc0
	}
	if c >= ascA && c <= ascZ {
		return c - ascA + 10
	}
	return 0
}

const strDigits = "0123456789ABCDEF"

func openfile(fn, mode string) int {
	switch strings.ToUpper(fn) {
	case "SYSIN":
		return sysin
	case "SYSPRINT":
		return sysprint
	}
	var f *os.File
	var err error
	if mode == "r" {
		f, err = os.Open(fn)
		if err != nil {
			lfn := strings.ToLower(fn)
			if lfn == fn {
				return 0
			}
			f, err = os.Open(lfn)
			if err != nil {
				return 0
			}
		}
	} else {
		f, err = os.Create(fn)
		if err != nil {
			return 0
		}
	}
	h := nextHandle
	nextHandle++
	fileHandles[h] = f
	inputReaders[h] = bufio.NewReader(f)
	return h
}

func findinput(fnBCPL int) int  { return openfile(cstr(fnBCPL), "r") }
func findoutput(fnBCPL int) int { return openfile(cstr(fnBCPL), "w") }

func endread() {
	if cis != 1 {
		if f, ok := fileHandles[cis]; ok {
			_ = f.Close()
			delete(fileHandles, cis)
			delete(inputReaders, cis)
		}
	}
	cis = sysin
}

func endwrite() {
	if cos != 2 {
		if f, ok := fileHandles[cos]; ok {
			_ = f.Close()
			delete(fileHandles, cos)
		}
	}
	cos = sysprint
}

func rdch() int {
	r, ok := inputReaders[cis]
	if !ok {
		return endStreamCh
	}
	c, err := r.ReadByte()
	if err != nil {
		return endStreamCh
	}
	if c == byte(ascCR) {
		return ascLF
	}
	return int(c)
}

func wrch(c int) {
	if c == ascLF {
		newline()
		return
	}
	if cos == 2 {
		_, _ = os.Stdout.Write([]byte{byte(c & 0xFF)})
		return
	}
	if f, ok := fileHandles[cos]; ok {
		_, _ = f.Write([]byte{byte(c & 0xFF)})
	}
}

func newline() {
	if cos == 2 {
		_, _ = os.Stdout.Write([]byte{'\n'})
		return
	}
	if f, ok := fileHandles[cos]; ok {
		_, _ = f.Write([]byte{'\n'})
	}
}

func writes(sPtr int) {
	byteIdx := sPtr * 2
	length := getByte(byteIdx)
	for i := 0; i < length; i++ {
		wrch(getByte(byteIdx + 1 + i))
	}
}

func writed(n, d int) {
	s := fmt.Sprintf("%d", n)
	for len(s) < d {
		s = " " + s
	}
	for _, c := range s {
		wrch(int(c))
	}
}

func writen(n int) { writed(n, 0) }

func readn() int {
	c := rdch()
	for c == ascSpace || c == ascLF || c == ascTab {
		c = rdch()
	}
	neg := c == ascMinus
	if neg || c == ascPlus {
		c = rdch()
	}
	total := 0
	for c >= asc0 && c <= asc9 {
		total = total*10 + (c - asc0)
		c = rdch()
	}
	m[k71Terminator] = c
	if neg {
		return -total
	}
	return total
}

func writeoct(n, d int) {
	n &= 0xFFFF
	if d > 1 {
		writeoct(n>>3, d-1)
	}
	wrch(int(strDigits[n&7]))
}

func writehex(n, d int) {
	n &= 0xFFFF
	if d > 1 {
		writehex(n>>4, d-1)
	}
	wrch(int(strDigits[n&15]))
}

func writef(vPtr int) {
	fmtPtr := m[vPtr]
	vPtr++
	byteIdx := fmtPtr * 2
	length := getByte(byteIdx)
	ss := 1
	for ss <= length {
		c := getByte(byteIdx + ss)
		ss++
		if c != ascPercent {
			wrch(c)
			continue
		}
		c = getByte(byteIdx + ss)
		ss++
		switch c {
		case ascS:
			writes(m[vPtr])
			vPtr++
		case ascC:
			wrch(m[vPtr])
			vPtr++
		case ascO:
			n := muGet(vPtr)
			vPtr++
			d := decval(getByte(byteIdx + ss))
			ss++
			writeoct(n, d)
		case ascX:
			n := muGet(vPtr)
			vPtr++
			d := decval(getByte(byteIdx + ss))
			ss++
			writehex(n, d)
		case ascI:
			n := m[vPtr]
			vPtr++
			d := decval(getByte(byteIdx + ss))
			ss++
			writed(n, d)
		case ascN:
			writen(m[vPtr])
			vPtr++
		default:
			wrch(c)
		}
	}
}

func packstring(vPtr, sPtr int) int {
	length := m[vPtr]
	n := length / bytesPerWord
	m[sPtr+n] = 0
	byteDest := sPtr * 2
	for i := 0; i <= length; i++ {
		setByte(byteDest+i, m[vPtr+i]&0xFF)
	}
	return n
}

func unpackstring(sPtr, vPtr int) {
	byteSrc := sPtr * 2
	length := getByte(byteSrc)
	for i := 0; i <= length; i++ {
		m[vPtr+i] = getByte(byteSrc + i)
	}
}

func stw(w int) {
	m[lomem] = s16(w)
	lomem++
	cp = 0
}

func stc(c int) {
	if cp == 0 {
		stw(0)
	}
	byteAddr := (lomem-1)*2 + cp
	setByte(byteAddr, c)
	cp++
	if cp == bytesPerWord {
		cp = 0
	}
}

func rch() {
	ch = rdch()
	for ch == ascSlash {
		for ch != ascLF && ch != endStreamCh {
			ch = rdch()
		}
		for ch == ascLF {
			ch = rdch()
		}
	}
}

func rdn() int {
	total := 0
	neg := ch == ascMinus
	if neg {
		rch()
	}
	for ch >= asc0 && ch <= asc9 {
		total = total*10 + (ch - asc0)
		rch()
	}
	if neg {
		return -total
	}
	return total
}

func labref(n, a int) {
	k := m[labvOffset+n]
	if k < 0 {
		k = -k
	} else {
		m[labvOffset+n] = a
	}
	m[a] = s16(m[a] + k)
}

func halt(msg string, n ...int) {
	if len(n) > 0 {
		fmt.Fprintf(os.Stderr, "%s #%d\n", msg, n[0])
	} else {
		fmt.Fprintln(os.Stderr, msg)
	}
	os.Exit(1)
}

func assemble() {
	for i := 0; i < labvCount; i++ {
		m[labvOffset+i] = 0
	}
	cp = 0
	rch()
	for {
		if ch >= asc0 && ch <= asc9 {
			n := rdn()
			k := m[labvOffset+n]
			if k < 0 {
				halt("DUPLICATE LABEL", n)
			}
			for k > 0 {
				tmp := m[k]
				m[k] = lomem
				k = tmp
			}
			m[labvOffset+n] = -lomem
			cp = 0
			continue
		}

		if ch == endStreamCh {
			return
		}
		if ch == ascDollar || ch == ascSpace || ch == ascLF {
			rch()
			continue
		}

		n := -1
		switch ch {
		case ascL:
			n = f0L
		case ascS:
			n = f1S
		case 'A':
			n = f2A
		case ascJ:
			n = f3J
		case ascT:
			n = f4T
		case ascF:
			n = f5F
		case ascK:
			n = f6K
		case ascX:
			n = f7X
		case ascC:
			rch()
			stc(rdn())
			continue
		case ascD:
			rch()
			if ch == ascL {
				rch()
				stw(0)
				labref(rdn(), lomem-1)
			} else {
				stw(rdn())
			}
			continue
		case ascG:
			rch()
			g := rdn()
			if ch != ascL {
				halt("BAD CODE AT P", lomem)
			}
			rch()
			m[g] = 0
			labref(rdn(), g)
			continue
		case ascZ:
			for i := 0; i < labvCount; i++ {
				if m[labvOffset+i] > 0 {
					halt("UNSET LABEL", i)
				}
			}
			for i := 0; i < labvCount; i++ {
				m[labvOffset+i] = 0
			}
			cp = 0
			rch()
			continue
		default:
			halt("BAD CH", ch)
		}

		rch()
		if ch == ascI {
			n |= fiBit
			rch()
		}
		if ch == ascP {
			n |= fpBit
			rch()
		}
		if ch == ascG {
			rch()
		}

		if ch == ascL {
			rch()
			stw(n | fdBit)
			stw(0)
			labref(rdn(), lomem-1)
			continue
		}
		d := rdn()
		if (d & fnMask) == d {
			stw(n | (d << fnBits))
		} else {
			stw(n | fdBit)
			stw(d)
		}
	}
}

func allocvec(n int) int {
	if n <= 0 {
		return 0
	}
	prev := 0
	cur := vecfree
	for cur != 0 {
		sz := m[cur]
		nxt := m[cur+1]
		if sz >= n {
			if prev != 0 {
				m[prev+1] = nxt
			} else {
				vecfree = nxt
			}
			return cur + 2
		}
		prev = cur
		cur = nxt
	}
	total := n + 2
	h := himem - total + 1
	if h <= lomem {
		return 0
	}
	m[h] = n
	m[h+1] = 0
	himem = h - 1
	return h + 2
}

func freevec(p int) {
	if p == 0 {
		return
	}
	h := p - 2
	m[h+1] = vecfree
	vecfree = h
}

func interpret() int {
	pc := progStart
	sp := lomem
	a := 0
	b := 0

	for {
		w := muGet(pc)
		pc++
		var d int
		if (w & fdBit) != 0 {
			d = m[pc]
			pc++
		} else {
			d = w >> fnBits
		}
		if (w & fpBit) != 0 {
			d = s16(d + sp)
		}
		if (w & fiBit) != 0 {
			d = m[d]
		}
		fn := w & 7
		switch fn {
		case f0L:
			b, a = a, d
		case f1S:
			m[d] = a
		case f2A:
			a = s16(a + d)
		case f3J:
			pc = d
		case f4T:
			if a != 0 {
				pc = d
			}
		case f5F:
			if a == 0 {
				pc = d
			}
		case f6K:
			d = s16(d + sp)
			if a < progStart {
				vPtr := d + 2
				switch a {
				case 1:
				case 2:
					m[sp] = 0
					m[sp+1] = progStart + 2
					pc = a
				case 3, 4:
				case k11SelectInput:
					cis = m[vPtr]
				case k12SelectOutput:
					cos = m[vPtr]
				case k13RDCH:
					a = rdch()
				case k14WRCH:
					wrch(m[vPtr])
				case k16Input:
					a = cis
				case k17Output:
					a = cos
				case k30STOP:
					return m[vPtr]
				case k31LEVEL:
					a = sp
				case k32LONGJUMP:
					sp = m[vPtr]
					pc = m[vPtr+1]
				case k40APTOVEC:
					b = d + m[vPtr+1] + 1
					m[b] = sp
					m[b+1] = pc
					m[b+2] = d
					m[b+3] = m[vPtr+1]
					sp = b
					pc = m[vPtr]
				case k41FindOutput:
					a = findoutput(m[vPtr])
				case k42FindInput:
					a = findinput(m[vPtr])
				case k46EndRead:
					endread()
				case k47EndWrite:
					endwrite()
				case k60Writes:
					writes(m[vPtr])
				case k62Writen:
					writen(m[vPtr])
				case k63Newline:
					newline()
				case k64Newpage:
					wrch(ascFF)
				case k66PackString:
					a = packstring(m[vPtr], m[vPtr+1])
				case k67UnpackString:
					unpackstring(m[vPtr], m[vPtr+1])
				case k68Writed:
					writed(m[vPtr], m[vPtr+1])
				case k70Readn:
					a = readn()
				case k75Writehex:
					writehex(m[vPtr]&0xFFFF, m[vPtr+1])
				case k76Writef:
					writef(vPtr)
				case k77Writeoct:
					writeoct(m[vPtr]&0xFFFF, m[vPtr+1])
				case k85Getbyte:
					base := m[vPtr] * 2
					offset := m[vPtr+1]
					a = getByte(base + offset)
				case k86Putbyte:
					base := m[vPtr] * 2
					offset := m[vPtr+1]
					setByte(base+offset, m[vPtr+2])
				case k87Getvec:
					a = allocvec(m[vPtr])
				case k88Freevec:
					freevec(m[vPtr])
				case k90Changeco:
					arg := m[vPtr]
					cptr := m[vPtr+1]
					currcoAddr := m[vPtr+2]
					if cptr <= 0 || cptr+6 >= wordCount {
						halt("BAD CHANGECO C", 0)
					}
					if currcoAddr < 0 || currcoAddr >= wordCount {
						halt("BAD CURRCO", 0)
					}
					currco := m[currcoAddr]
					if coDebug {
						fmt.Fprintf(os.Stderr, "CHANGECO enter: arg=%d cptr=%d currco_addr=%d currco=%d sp=%d pc=%d\n", arg, cptr, currcoAddr, currco, sp, pc)
						fmt.Fprintf(os.Stderr, "  cptr fields: [0]sp=%d [1]pc=%d [2]parent=%d [3]next=%d [4]fn=%d [5]size=%d [6]self=%d\n",
							m[cptr], m[cptr+1], m[cptr+2], m[cptr+3], m[cptr+4], m[cptr+5], m[cptr+6])
					}
					if currco != 0 {
						if currco < 0 || currco+1 >= wordCount {
							halt("BAD CURRCO VAL", 0)
						}
						m[currco] = sp
						m[currco+1] = pc
						if coDebug {
							fmt.Fprintf(os.Stderr, "  saved currco[%d]: sp=%d pc=%d\n", currco, sp, pc)
						}
					}
					m[currcoAddr] = cptr
					sp = m[cptr]
					pc = m[cptr+1]
					if coDebug {
						fmt.Fprintf(os.Stderr, "CHANGECO exit: new_sp=%d new_pc=%d a=%d\n", sp, pc, arg)
					}
					if sp >= wordCount || sp < progStart {
						halt("BAD CHANGECO SP", sp)
					}
					if pc >= wordCount || pc < progStart {
						halt("BAD CHANGECO PC", pc)
					}
					a = arg
				default:
					halt("UNKNOWN CALL", a)
				}
			} else {
				m[d] = sp
				m[d+1] = pc
				sp = d
				pc = a
			}
		case f7X:
			switch d {
			case 1:
				a = m[a]
			case 2:
				a = s16(-a)
			case 3:
				a = s16(^a)
			case 4:
				pc = m[sp+1]
				sp = m[sp]
			case 5:
				a = s16(b * a)
			case 6:
				if a != 0 {
					sign := 1
					if (b < 0) != (a < 0) {
						sign = -1
					}
					a = sign * (abs(b) / abs(a))
				}
			case 7:
				if a != 0 {
					if b < 0 {
						a = -(abs(b) % abs(a))
					} else {
						a = abs(b) % abs(a)
					}
				}
			case 8:
				a = s16(b + a)
			case 9:
				a = s16(b - a)
			case 10:
				if b == a {
					a = -1
				} else {
					a = 0
				}
			case 11:
				if b != a {
					a = -1
				} else {
					a = 0
				}
			case 12:
				if b < a {
					a = -1
				} else {
					a = 0
				}
			case 13:
				if b >= a {
					a = -1
				} else {
					a = 0
				}
			case 14:
				if b > a {
					a = -1
				} else {
					a = 0
				}
			case 15:
				if b <= a {
					a = -1
				} else {
					a = 0
				}
			case 16:
				a = s16(b << a)
			case 17:
				a = s16((b & 0xFFFF) >> a)
			case 18:
				a = s16(b & a)
			case 19:
				a = s16(b | a)
			case 20:
				a = s16(b ^ a)
			case 21:
				a = s16(b ^ (^a))
			case 22:
				return 0
			case 23:
				vIdx := pc
				cnt := m[vIdx]
				vIdx++
				pc = m[vIdx]
				vIdx++
				for cnt > 0 {
					if a == m[vIdx] {
						pc = m[vIdx+1]
						break
					}
					vIdx += 2
					cnt--
				}
			default:
				halt("UNKNOWN EXEC", d)
			}
		}
	}
}

func abs(v int) int {
	if v < 0 {
		return -v
	}
	return v
}

func loadcode(fn string) int {
	f := openfile(fn, "r")
	if f != 0 {
		cis = f
		assemble()
		endread()
	}
	return f
}

func initRuntime() {
	for i := 0; i < progStart; i++ {
		m[i] = i
	}
	lomem = progStart
	himem = wordCount - labvCount - 1
	vecfree = 0

	stw(f0L | fiBit | (1 << fnBits))
	stw(f6K | (2 << fnBits))
	stw(f7X | (22 << fnBits))

	cis = 1
	cos = 2
	sysin = 1
	sysprint = 2
	fileHandles[1] = os.Stdin
	fileHandles[2] = os.Stdout
	inputReaders[1] = bufio.NewReader(os.Stdin)
}

func pipeinput(fn string) {
	f := openfile(fn, "r")
	if f == 0 {
		halt("NO INPUT")
	}
	cis = f
	sysin = f
}

func pipeoutput(fn string) {
	f := openfile(fn, "w")
	if f == 0 {
		halt("NO OUTPUT")
	}
	cos = f
	sysprint = f
}

func main() {
	initRuntime()
	if len(os.Args) < 2 {
		fmt.Println("USAGE: go run . ICFILE [...] [-iINPUT] [-oOUTPUT]")
		os.Exit(0)
	}
	for _, arg := range os.Args[1:] {
		if strings.HasPrefix(arg, "-") {
			if strings.HasPrefix(arg, "-i") {
				pipeinput(arg[2:])
			} else if strings.HasPrefix(arg, "-o") {
				pipeoutput(arg[2:])
			} else {
				halt("INVALID OPTION")
			}
		} else {
			if loadcode(arg) == 0 {
				// Keep compatibility with common workflow where paths are relative to cwd.
				if !filepath.IsAbs(arg) {
					if loadcode(filepath.Clean(arg)) == 0 {
						halt("NO ICFILE")
					}
				} else {
					halt("NO ICFILE")
				}
			}
		}
	}
	os.Exit(interpret())
}

