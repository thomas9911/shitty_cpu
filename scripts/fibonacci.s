mov r5 #1234 ; find first fibonacci larger than 1234
mov r1 #1
mov r0 #1
start:
  cmp r0 r5
  bge :stop
  mov r2 r0
  add r0 r1
  mov r1 r2
  b :start
stop:
