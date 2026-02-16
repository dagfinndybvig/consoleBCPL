// Test 6: DELETECO
// Tests that coroutines can be deleted and memory freed.

GET "CORO_LIB.b"

LET WORKER(ARG) = VALOF
$( WRITES("worker got: "); WRITEN(ARG); NEWLINE()
   RESULTIS ARG * 2
$)

LET START() BE
$( LET C1 = ?
   LET C2 = ?
   LET C3 = ?
   LET V = ?
   LET OK = TRUE
   WRITES("Test 6: DELETECO*N")
   INITCO()

   // Create three coroutines
   C1 := CREATECO(WORKER, 200)
   C2 := CREATECO(WORKER, 200)
   C3 := CREATECO(WORKER, 200)
   IF C1 = 0 | C2 = 0 | C3 = 0 DO
   $( WRITES("CREATECO FAILED*N")
      STOP(1)
   $)
   WRITES("created 3 coroutines*N")

   // Use C1
   V := CALLCO(C1, 5)
   WRITES("C1 returned: "); WRITEN(V); NEWLINE()
   IF V NE 10 DO OK := FALSE

   // Delete C1
   UNLESS DELETECO(C1) DO
   $( WRITES("DELETECO C1 FAILED*N")
      OK := FALSE
   $)
   WRITES("deleted C1*N")

   // Use C2
   V := CALLCO(C2, 7)
   WRITES("C2 returned: "); WRITEN(V); NEWLINE()
   IF V NE 14 DO OK := FALSE

   // Delete C2 and C3
   UNLESS DELETECO(C2) DO
   $( WRITES("DELETECO C2 FAILED*N")
      OK := FALSE
   $)
   UNLESS DELETECO(C3) DO
   $( WRITES("DELETECO C3 FAILED*N")
      OK := FALSE
   $)
   WRITES("deleted C2 and C3*N")

   TEST OK
   THEN WRITES("Test 6 PASSED*N")
   ELSE WRITES("Test 6 FAILED*N")
$)
