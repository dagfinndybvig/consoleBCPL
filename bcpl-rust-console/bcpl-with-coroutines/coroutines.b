GLOBAL $(
   CURRCO:500;
   COLIST:501
$)

LET ABORT(N) = STOP(N)

LET INITCO() BE
$( IF CURRCO=0 THEN
   $( LET C = GETVEC(7)
      IF C=0 DO ABORT(200)
      C!0 := LEVEL()
      C!1 := 0
      C!2 := 0
      C!3 := COLIST
      C!4 := 0
      C!5 := 0
      C!6 := C
      COLIST := C
      CURRCO := C
   $)
$)

AND COROENTRY() BE
$( LET C = CURRCO
   LET F = C!4
   LET ARG = COWAIT(C)
   WHILE TRUE DO
   $( C := F(ARG)
      ARG := COWAIT(C)
   $)
$)

AND CREATECO(F, SIZE) = VALOF
$( LET C = GETVEC(SIZE + 7)
   LET SP0 = 0
   IF C=0 RESULTIS 0
   SP0 := C + 7

   C!0 := SP0            // saved stack pointer
   C!1 := COROENTRY      // saved pc (entry)
   C!2 := 0              // parent link
   C!3 := COLIST         // coroutine list link
   C!4 := F              // main procedure
   C!5 := SIZE           // stack size
   C!6 := C              // coroutine pointer

   SP0!0 := 0
   SP0!1 := 0

   COLIST := C
   RESULTIS C
$)

AND DELETECO(CPTR) = VALOF
$( LET A = @COLIST
   WHILE !A NE 0 & !A NE CPTR DO A := !A+3

   IF !A=0 RESULTIS FALSE
   UNLESS CPTR!2=0 DO ABORT(112)

   !A := CPTR!3
   FREEVEC(CPTR)
   RESULTIS TRUE
$)

AND CALLCO(CPTR, A) = VALOF
$( UNLESS CPTR!2=0 DO ABORT(110)
   CPTR!2 := CURRCO
   RESULTIS CHANGECO(A, CPTR, @CURRCO)
$)

AND COWAIT(A) = VALOF
$( LET PARENT = CURRCO!2
   CURRCO!2 := 0
   IF PARENT=0 DO ABORT(111)
   RESULTIS CHANGECO(A, PARENT, @CURRCO)
$)

AND RESUMECO(CPTR, A) = VALOF
$( LET PARENT = CURRCO!2
   CURRCO!2 := 0
   IF CPTR=CURRCO RESULTIS A
   UNLESS CPTR!2=0 DO ABORT(111)
   CPTR!2 := PARENT
   RESULTIS CHANGECO(A, CPTR, @CURRCO)
$)
