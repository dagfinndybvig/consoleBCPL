GET "LIBHDR"
GET "coroutines"

LET Worker(Arg) = VALOF
$( LET V = Arg
   $(
      WRITES("worker got ")
      WRITEN(V)
      NEWLINE()
      V := Cowait(V+1)
   $) REPEAT
$)

LET START() BE
$( Initco()
   LET C = Createco(Worker, 200)
   LET V = 1
   V := Callco(C, V)
   WRITES("main got ")
   WRITEN(V)
   NEWLINE()
   V := Resumeco(C, 10)
   WRITES("main got ")
   WRITEN(V)
   NEWLINE()
   Deleteco(C)
$)
