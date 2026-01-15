GET "LIBHDR"

MANIFEST $(
    LOWER_A='a';
    LOWER_Z='z';
    CASE_DELTA='a'-'A'
$)

LET START() BE
$( LET C = ?

    UNTIL C=ENDSTREAMCH DO
    $(
        C := RDCH()
        IF NOT C=ENDSTREAMCH
        $(
            IF C>=LOWER_A & C<=LOWER_Z C := C - CASE_DELTA
            WRCH(C)
        $)
    $)
$)
