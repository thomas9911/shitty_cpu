data: db ""
    mov r5 #10
    mov r1 #0
random_loop:
    func :getrandom
    add r1 #1
    cmp r1 r5
    bl :random_loop
    mov r1 #0
print_loop:
    pop r0
    ; print random as lowercase letters
    mod r0 #26
    add r0 #97
    mov [:data] r0
    push :data
    func :print
    add r1 #1
    cmp r1 r5
    bl :print_loop
