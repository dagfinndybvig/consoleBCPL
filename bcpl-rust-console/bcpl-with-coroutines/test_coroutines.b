GET "LIBHDR"
GET "coroutines"

LET WORKER(ARG) = VALOF
$( 
   $(
      LET VAL = ?
      WRITES("worker got ")
      WRITEF("%I", ARG)
      NEWLINE()
      VAL := COWAIT(ARG+1)
   $) REPEAT
$)

LET START() BE
$( 
   LET C = ?
   LET V = ?
   INITCO()
   C := CREATECO(WORKER, 5000)  
   V := CALLCO(C, 1)
   WRITES("main got ")
   WRITEF("%I", V)
   NEWLINE()
   // V := RESUMECO(C, 10)
   WRITES("main got ")
   WRITEF("%I", V)
   NEWLINE()
   DELETECO(C)
$)
