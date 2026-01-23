GET "LIBHDR"
GET "coroutines"

LET WORKER(ARG) = VALOF
$( WRITES("Coroutines work")
   NEWLINE()
   ARG := COWAIT(ARG+1)
   WRITES("Coroutines work")
   NEWLINE()
   //WRITEN(ARG)
   ARG := COWAIT(ARG+1)
   RESULTIS ARG+1
$)

LET START() BE
$( LET C = 0
   LET V = 0
   INITCO()
   C := CREATECO(WORKER, 5000)
   V := CALLCO(C, 0)

   V := CALLCO(C, 10)
   WRITES("Coroutines work")
   NEWLINE()
   //WRITEN(V)

   V := CALLCO(C, 20)
   WRITES("Coroutines work")
   NEWLINE()

   V := CALLCO(C, 30)
   IF V NE 31 DO
   $( WRITES("RETURN MISMATCH")
      NEWLINE()
      STOP(999)
   $)

   WRITES("Lines: 5")
   NEWLINE()
   DELETECO(C)
$)
