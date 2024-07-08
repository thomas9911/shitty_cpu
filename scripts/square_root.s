mov r6 #529 ; input
mov r5 #0 ; counter
mov r0 r6 ; initial value 
mov r1 #0
mov r2 #0
mov r3 #0
start:
    cmp r5 #8
    bge :stop
    ; 2x
    mov r2 r0
    mul r2 #2
    ; x^2
    mov r3 r0
    mul r3 r3
    ; x^2 - r6
    sub r3 r6
    ; (x^2 - r6) / 2x
    div r3 r2
    ; x - (x^2 - r6) / 2x
    sub r0 r3
    ; increase counter
    add r5 #1
    b :start
stop:
    sub r0 #1
