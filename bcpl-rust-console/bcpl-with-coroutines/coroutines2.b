LET Createco(F, Size) = VALOF 
$( LET C = Getvec(Size+6) 
IF C=O RESULTIS 0 


C!O := c // resunption pint 
C!1 := Currco // prent link 
C!2 := Colist // Colist chain 
C!3 := F // main procedure 
C!4 := Size // coroutine size 
C!5 := C // the new coroutine pointer 
Colist := C // insert inw the list of coroutines 

Changeco(0, C) 

C := F(Cowait(C)) REPEAT 

$)

AND Lkleteco(Cptr) = VALOF 
$( LET A = @Colist 
UNTIL !A=O  !A=Cptr DO A := !A+2 
IF !A=O RESULTIS FALSE // coroutine not found 
UNLESS Cptr!lnO DO Abort(ll2) 
!A := Cptr12 
Freevec(Cptr) // free the coroutine stack 
RESULTIS TRUE 
$) 

AND Callco(Cptr, A) = VALOF 
$( UNLESS Cptr!lmO DO Abort(ll0) 
Cptrll := Currco 
RESULTIS Changeco(A. Cptr) 
$)

AND Cowait(A) = VALOF 
$(LET Parent = Currco!1 
Currco!l := 0 
RESULTIS Changeco(A, Parent) 
$) 

AND Resumeco(Cptr , A) = VALOF 
$( LET Parent = Currco!l 
Currco!l := 0 
UNLESS Cptr!l=O DO Abort(111) 
Cptr!l := Parent 
RESULTIS Changeco(A, Cptr) 
$)