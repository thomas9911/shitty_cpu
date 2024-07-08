; calculation of 9!
mov r5 #9
mov r0 #1
mov r1 #1
start:
  cmp r1 r5
  bg :stop
  mul r0 r1
  add r1 #1
  b :start
stop:
