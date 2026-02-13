// Test 3: Single CALLCO / COWAIT round-trip
// Main calls worker with value 42, worker adds 1 and COWAITs back.

GET "CORO_LIB.b"

LET WORKER(ARG) = VALOF
$( WRITES("worker got: "); WRITEN(ARG); NEWLINE()
   RESULTIS ARG + 1
$)

LET START() BE
$( LET C = ?
   LET V = ?
   WRITES("Test 3: CALLCO/COWAIT round-trip*N")
   INITCO()

   C := CREATECO(WORKER, 200)
   IF C = 0 DO
   $( WRITES("CREATECO FAILED*N")
      STOP(1)
   $)
   WRITES("created coroutine: "); WRITEN(C); NEWLINE()

   V := CALLCO(C, 42)
   WRITES("main got back: "); WRITEN(V); NEWLINE()

   TEST V = 43
   THEN WRITES("Test 3 PASSED*N")
   ELSE WRITES("Test 3 FAILED*N")

   DELETECO(C)
$)
