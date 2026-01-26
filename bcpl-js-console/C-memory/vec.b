GET "LIBHDR"

LET START() = VALOF
$(
  LET N = 50
  LET V = ?  
  LET I = 0

  V := GETVEC(N)
  
  IF V = 0 $( 
    WRITES("GETVEC FAILED")
    NEWLINE()
    STOP(1) $)
 
  FOR I = 0 TO N-1 DO
  $(
    V!I := (I + 1) * 10
  $)

  FOR I = 0 TO N-1 DO
  $(
    WRITES("V!")
    WRITEN(I)
    WRITES(" = ")
    WRITEN(V!I)
    NEWLINE()
  $)

  FREEVEC(V)

  WRITES("TEST PASSED"); NEWLINE()
  RESULTIS 0
$)