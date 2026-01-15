// OLD-STYLE SYNTAX

// SECTION "COINS" DOES NOT COMPILE IN OLD
 
GET "LIBHDR"
 
LET COINS(SUM) = C(SUM, (TABLE 200, 100, 50, 20, 10, 5, 2, 1))
 
AND C(SUM, T) = SUM<0 -> 0,
                SUM=0 | !T=1 -> 1,
                C(SUM, T+1) + C(SUM-!T, T)
 
LET START() = VALOF
$( WRITES("COINS PROBLEM*N")
 
  T(0)
  T(1)
  T(2)
  T(5)
  T(21)
  T(100)
  T(200)
  RESULTIS 0
$)
 
AND T(N) BE WRITEF("SUM = %I3  NUMBER OF WAYS = %I6*N", N, COINS(N))
