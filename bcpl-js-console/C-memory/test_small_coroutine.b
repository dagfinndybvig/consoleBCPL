GET "corns.b"

LET WORKER(ARG) = VALOF
$( LET V = ARG
   WRITES("worker start")
   NEWLINE()
   WRITES("worker first yield")
   NEWLINE()
   V := COWAIT(V+1)
   WRITES("worker resumed")
   NEWLINE()
   RESULTIS V
$)

LET START() BE
$( INITCO()
   LET C = CREATECO(WORKER, 100)
   LET V = CALLCO(C, 1)
   WRITES("main got ")
   WRITEN(V)
   NEWLINE()
   V := CALLCO(C, 10)
   WRITES("main got ")
   WRITEN(V)
   NEWLINE()
   DELETECO(C)
$)
