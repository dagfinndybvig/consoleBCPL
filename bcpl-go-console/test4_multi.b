// Test 4: Multiple CALLCO / COWAIT exchanges
// Worker yields multiple times before finishing.

GET "CORO_LIB.b"

LET WORKER(ARG) = VALOF
$( LET V = ARG
   WRITES("worker start, got: "); WRITEN(V); NEWLINE()

   // First yield
   V := COWAIT(V + 10)
   WRITES("worker resumed, got: "); WRITEN(V); NEWLINE()

   // Second yield
   V := COWAIT(V + 20)
   WRITES("worker resumed again, got: "); WRITEN(V); NEWLINE()

   // Final return (via COROENTRY's COWAIT of the result)
   RESULTIS V + 30
$)

LET START() BE
$( LET C = ?
   LET V = ?
   LET OK = TRUE
   WRITES("Test 4: Multiple yields*N")
   INITCO()

   C := CREATECO(WORKER, 200)
   IF C = 0 DO
   $( WRITES("CREATECO FAILED*N")
      STOP(1)
   $)

   // First call: pass 1, expect 1+10=11
   V := CALLCO(C, 1)
   WRITES("main got: "); WRITEN(V); NEWLINE()
   IF V NE 11 DO OK := FALSE

   // Second call: pass 100, expect 100+20=120
   V := CALLCO(C, 100)
   WRITES("main got: "); WRITEN(V); NEWLINE()
   IF V NE 120 DO OK := FALSE

   // Third call: pass 200, expect 200+30=230
   V := CALLCO(C, 200)
   WRITES("main got: "); WRITEN(V); NEWLINE()
   IF V NE 230 DO OK := FALSE

   TEST OK
   THEN WRITES("Test 4 PASSED*N")
   ELSE WRITES("Test 4 FAILED*N")

   DELETECO(C)
$)
