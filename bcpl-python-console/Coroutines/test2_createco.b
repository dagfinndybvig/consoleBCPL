// Test 2: INITCO + CREATECO + immediate COWAIT
// Verifies that creating a coroutine works — COROENTRY runs and
// immediately COWAITs back to the creator.

GET "LIBHDR"
GET "CORHDR"

LET MYFUNC(ARG) = VALOF
$( WRITES("MYFUNC should NOT run yet*N")
   RESULTIS ARG
$)

LET START() BE
$( LET C = ?
   WRITES("Test 2: CREATECO + suspend*N")
   INITCO()
   WRITES("INITCO done, CURRCO="); WRITEN(CURRCO); NEWLINE()
   DUMPCO(CURRCO)

   C := CREATECO(MYFUNC, 200)
   IF C = 0 DO
   $( WRITES("CREATECO FAILED*N")
      STOP(1)
   $)
   WRITES("CREATECO returned: "); WRITEN(C); NEWLINE()
   DUMPCO(C)
   WRITES("Test 2 PASSED*N")
$)
