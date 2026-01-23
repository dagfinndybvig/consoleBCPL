GET "LIBHDR"
GET "coroutines"

LET WORKER(ARG) = VALOF
$( 
   $(
      LET VAL = ?
      WRITES("worker got ")
      WRITEN(ARG)
      NEWLINE()
      VAL := COWAIT(ARG+1)
   $) REPEAT
$)

LET START() BE
$( 
   LET C = ?
   LET V = ?
   INITCO()
   C := CREATECO(WORKER, 500)  
   V := CALLCO(C, 1)
   WRITES("main got ")
   WRITEN(V)
   NEWLINE()
   V := RESUMECO(C, 10)
   WRITES("main got ")
   WRITEF("%I", V)
   NEWLINE()
   DELETECO(C)
$)
