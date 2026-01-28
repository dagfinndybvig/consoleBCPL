// Testing the C-version with GETVEC/FREEVEC
// Should also work on the JS version based on that
// In the bcpls-c-console directory:
// ./C_compile testvec.b
// ./JS_compile testvec.b


GET "LIBHDR"

LET START() = VALOF
$(
  LET N = 5
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