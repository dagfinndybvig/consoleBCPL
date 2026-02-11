GET "LIBHDR"

// Test 1: Verify build pipeline and basic GETVEC/FREEVEC work.
// This does NOT test coroutines yet - just proves the interpreter
// and compiler toolchain are working correctly in new_coroutines/.

LET START() BE
$( LET V = GETVEC(10)
   WRITES("Test 1: Build pipeline check*N")
   WRITES("GETVEC(10) returned: ")
   WRITEN(V)
   NEWLINE()
   IF V = 0 DO
   $( WRITES("GETVEC FAILED*N")
      STOP(1)
   $)
   V!0 := 42
   V!1 := 99
   WRITES("V!0 = "); WRITEN(V!0); NEWLINE()
   WRITES("V!1 = "); WRITEN(V!1); NEWLINE()
   FREEVEC(V)
   WRITES("FREEVEC done*N")
   WRITES("LEVEL() = "); WRITEN(LEVEL()); NEWLINE()
   WRITES("Test 1 PASSED*N")
$)
