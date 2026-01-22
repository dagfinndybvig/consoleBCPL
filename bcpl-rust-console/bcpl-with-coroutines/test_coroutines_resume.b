GET "LIBHDR"
GET "coroutines"

LET START() BE
$( LET V = 0
   INITCO()
   V := RESUMECO(CURRCO, 7)
   IF V NE 7 DO $( WRITES("RESUME FAIL 1") ; NEWLINE() ; STOP(901) $)

   WRITES("Resume ok")
   NEWLINE()
$)
