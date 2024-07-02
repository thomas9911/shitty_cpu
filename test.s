    mov r0 #15
start:
    add r2 #1
    call :add_one
    mul r0 #7
    b :continue
add_one:
    add r0 #100
    ret
continue:
    add r0 #1000
    mov r1 #2000
    sub r1 r0
    cmp r2 #10
    bge :stop
    b :start
stop:
