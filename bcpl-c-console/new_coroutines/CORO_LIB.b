// CORO_LIB.b — Coroutine runtime library for consoleBCPL
//
// Provides: INITCO, CREATECO, DELETECO, CALLCO, COWAIT, RESUMECO, DUMPCO
//
// Control block layout (offsets from base pointer C):
//   C!0 = saved sp (for CHANGECO)
//   C!1 = saved pc (for CHANGECO)
//   C!2 = parent coroutine pointer (set by CALLCO, cleared by COWAIT)
//   C!3 = next pointer (COLIST linked list)
//   C!4 = function to run
//   C!5 = stack size
//   C!6 = self-pointer
//   C!7 onwards = coroutine's private stack area

GET "LIBHDR"
GET "CORHDR"

// DUMPCO — print coroutine control block fields (for debugging)
LET DUMPCO(C) BE
$( WRITES("CO @"); WRITEN(C); NEWLINE()
   WRITES("  [0]sp="); WRITEN(C!0)
   WRITES(" [1]pc="); WRITEN(C!1)
   WRITES(" [2]parent="); WRITEN(C!2)
   NEWLINE()
   WRITES("  [3]next="); WRITEN(C!3)
   WRITES(" [4]fn="); WRITEN(C!4)
   WRITES(" [5]size="); WRITEN(C!5)
   WRITES(" [6]self="); WRITEN(C!6)
   NEWLINE()
$)

// INITCO — initialize the main program as a coroutine
// Must be called before any other coroutine operations.
AND INITCO() BE
$( LET C = GETVEC(7)
   IF C = 0 DO STOP(200)
   C!0 := LEVEL()
   C!1 := 0
   C!2 := 0
   C!3 := 0
   C!4 := 0
   C!5 := 0
   C!6 := C
   COLIST := C
   CURRCO := C
$)

// COROENTRY — trampoline for new coroutines
// When a new coroutine is switched to for the first time,
// execution starts here. It immediately COWAITs back to the
// creator, then when resumed via CALLCO, runs the user function.
AND COROENTRY() BE
$( LET C = CURRCO
   LET F = C!4
   LET ARG = COWAIT(C)
   LET RESULT = F(ARG)
   COWAIT(RESULT)
   STOP(199)
$)

// CREATECO — create a new coroutine
// F = function the coroutine will run (takes one argument)
// SIZE = number of words for the coroutine's private stack
// Returns: pointer to the coroutine control block, or 0 on failure
AND CREATECO(F, SIZE) = VALOF
$( LET C = GETVEC(SIZE + 7)
   LET SP0 = ?
   IF C = 0 RESULTIS 0
   SP0 := C + 7

   // Initialize control block
   C!0 := SP0
   C!1 := COROENTRY
   C!2 := CURRCO                   // set parent to creator so COROENTRY's COWAIT works
   C!3 := COLIST
   C!4 := F
   C!5 := SIZE
   C!6 := C

   // Initialize synthetic stack frame at SP0
   // m[SP0] = saved sp (points to self = sentinel bottom of stack)
   // m[SP0+1] = saved pc (0 = sentinel, should never be returned to)
   SP0!0 := SP0
   SP0!1 := 0

   // Link into coroutine list
   COLIST := C

   // Switch to the new coroutine. COROENTRY will immediately COWAIT back.
   // This establishes valid sp/pc in the control block via CHANGECO's save.
   CHANGECO(0, C, @CURRCO)

   // We return here after COROENTRY's COWAIT suspends the new coroutine
   RESULTIS C
$)

// DELETECO — delete a coroutine and free its memory
// CPTR = pointer to coroutine control block
// Returns TRUE if successful, FALSE if not found
AND DELETECO(CPTR) = VALOF
$( LET A = @COLIST
   // Walk linked list to find and unlink CPTR
   WHILE !A NE 0 & !A NE CPTR DO A := (!A) + 3
   IF !A = 0 RESULTIS FALSE
   UNLESS CPTR!2 = 0 DO STOP(112)
   !A := CPTR!3
   FREEVEC(CPTR)
   RESULTIS TRUE
$)

// CALLCO — call a coroutine, passing it a value
// CPTR = target coroutine
// A = value to pass
// Returns: value passed back by COWAIT in the target
AND CALLCO(CPTR, A) = VALOF
$( UNLESS CPTR!2 = 0 DO STOP(110)
   CPTR!2 := CURRCO
   RESULTIS CHANGECO(A, CPTR, @CURRCO)
$)

// COWAIT — yield from current coroutine back to parent
// A = value to pass back to the caller
// Returns: value passed by the next CALLCO/RESUMECO
AND COWAIT(A) = VALOF
$( LET PARENT = CURRCO!2
   CURRCO!2 := 0
   IF PARENT = 0 DO STOP(111)
   RESULTIS CHANGECO(A, PARENT, @CURRCO)
$)

// RESUMECO — resume a coroutine, transferring the parent chain
// CPTR = target coroutine
// A = value to pass
// Returns: value passed back by COWAIT in the target
AND RESUMECO(CPTR, A) = VALOF
$( LET PARENT = CURRCO!2
   IF CPTR = CURRCO RESULTIS A
   CURRCO!2 := 0
   UNLESS CPTR!2 = 0 DO STOP(111)
   CPTR!2 := PARENT
   RESULTIS CHANGECO(A, CPTR, @CURRCO)
$)
