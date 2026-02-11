// Test 5: RESUMECO
// Tests parent-chain transfer via RESUMECO between two coroutines.

GET "CORO_LIB.b"

LET WORKERA(ARG) = VALOF
$( LET V = ARG
   WRITES("workera start, got: "); WRITEN(V); NEWLINE()
   V := COWAIT(V + 1)
   WRITES("workera resumed, got: "); WRITEN(V); NEWLINE()
   RESULTIS V + 1
$)

AND WORKERB(ARG) = VALOF
$( LET V = ARG
   WRITES("workerb start, got: "); WRITEN(V); NEWLINE()
   V := COWAIT(V + 100)
   WRITES("workerb resumed, got: "); WRITEN(V); NEWLINE()
   RESULTIS V + 100
$)

LET START() BE
$( LET CA = ?
   LET CB = ?
   LET V = ?
   LET OK = TRUE
   WRITES("Test 5: RESUMECO*N")
   INITCO()

   CA := CREATECO(WORKERA, 200)
   CB := CREATECO(WORKERB, 200)
   IF CA = 0 | CB = 0 DO
   $( WRITES("CREATECO FAILED*N")
      STOP(1)
   $)

   // Call workera with 10, expect 11
   V := CALLCO(CA, 10)
   WRITES("main got from A: "); WRITEN(V); NEWLINE()
   IF V NE 11 DO OK := FALSE

   // Call workerb with 20, expect 120
   V := CALLCO(CB, 20)
   WRITES("main got from B: "); WRITEN(V); NEWLINE()
   IF V NE 120 DO OK := FALSE

   // Resume workera with 50, it finishes, expect 51
   V := CALLCO(CA, 50)
   WRITES("main got from A (final): "); WRITEN(V); NEWLINE()
   IF V NE 51 DO OK := FALSE

   TEST OK
   THEN WRITES("Test 5 PASSED*N")
   ELSE WRITES("Test 5 FAILED*N")

   DELETECO(CA)
   DELETECO(CB)
$)
