.data
    value: .word 42

.text
.globl main
main:
    li $v0, 10
    syscall
