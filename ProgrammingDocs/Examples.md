# Cicada-16 Assembly Examples

This document provides a few short, well-commented programs to demonstrate basic programming concepts in Cicada-16 assembly.

---

### Example 1: Hello, World!

This program prints the string "Hello, World!" to the console using a hypothetical system call. It shows how to define data and loop through it.

```asm
.org 0x100
.define MSG_LEN 13

start:
    LDI R1, message     ; Load the address of the message into R1
    LDI R2, MSG_LEN     ; Load the length of the message into R2

print_loop:
    LD.b R3, (R1)+      ; Load a character from the message, post-increment address

    ; Hypothetical SYSCALL 0x01: Print the character in the lower byte of R3
    SYSCALL 0x01

    DEC R2              ; Decrement the length counter
    JRcc NZ, print_loop ; If not zero, loop to print the next character

end_loop:
    HALT                ; Halt the CPU

; --- Data ---
message:
    .byte 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21 ; "Hello, World!"
```

---

### Example 2: Sum of an Array

This program calculates the sum of a small array of 16-bit numbers. It demonstrates indexed addressing and looping.

```asm
.org 0x100

start:
    LDI R1, 0           ; R1 will be our loop counter (i)
    LDI R2, 0           ; R2 will hold the sum
    LDI R3, 5           ; R3 holds the number of elements to sum
    LDI R4, data_array  ; R4 holds the base address of the array

sum_loop:
    ; We can't do LD R0, (R4, R1) directly, so we calculate the address
    LD R0, R4           ; Copy base address to R0
    ADD R0, R1          ; Add index to get element address (R0 = R4 + R1)
    ADD R2, (R0)        ; Add the value from memory (at address in R0) to the sum in R2

    ADD R1, 2           ; Increment index by 2 (since each element is a 16-bit word)
    DEC R3              ; Decrement loop counter
    JRcc NZ, sum_loop   ; Loop if there are still elements left

end_loop:
    ; The sum is now in R2
    HALT

; --- Data ---
data_array:
    .word 10, 20, 30, 40, 50
```

---

### Example 3: Simple Subroutine

This example shows how to define and use a simple subroutine with `CALL` and `RET`. The subroutine multiplies `R0` by 2.

```asm
.org 0x100

start:
    LDI R0, 12          ; Load a value into the accumulator
    CALL multiply_by_two  ; Call the subroutine

    ; After returning, R0 will hold the value 24

    LDI R0, 5
    CALL multiply_by_two

    ; After returning, R0 will hold the value 10

    HALT

; --- Subroutine: multiply_by_two ---
; Multiplies the value in R0 by 2.
; Input: R0
; Output: R0
multiply_by_two:
    SHL R0              ; Logical shift left is equivalent to multiplying by 2
    RET                 ; Return to the caller
```

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
