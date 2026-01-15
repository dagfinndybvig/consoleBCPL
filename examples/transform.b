// Transform file to upper case
// Also replaces { } with $( $)
// To help migrate newer BCPL syntax to the classical one we are using

GET "LIBHDR"

LET START() BE
$( LET C = ?

    UNTIL C=ENDSTREAMCH DO
    $(
        C := RDCH()
        IF NOT C=ENDSTREAMCH
        $(
            IF C=123 $( WRCH(36) WRCH(40) $)
            IF C=125 $( WRCH(36) WRCH(41) $)
            IF NOT C=123 & NOT C=125
            $(
                IF C>=97 & C<=122 C := C - 32
                WRCH(C)
            $)
        $)
    $)
$)
