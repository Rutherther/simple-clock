# GDB script for loading and running programs in STM32 Blue Pill.
# This file used to be .gdbinit, which could not be autoloaded due to autoloading security in GDB.

# Send GDB commands to OpenOCD, which listens on port 3333.  Extend the timeout.
target ext :3333

# Disable all messages.
set verbose off
set complaints 0
set confirm off
set exec-done-display off
show exec-done-display
set trace-commands off
set debug displaced off
set debug expression 0
set debug frame 0
set debug infrun 0
set debug observer 0
set debug overload 0
set pagination off
set print address off
set print symbol-filename off
set print symbol off
set print pretty off
set print object off
set debug parser off
set debug remote 0

# Print demangled symbols by default.
set print asm-demangle on

# Enable ARM semihosting to show debug console output in OpenOCD console.
monitor arm semihosting enable

monitor reset init
monitor sleep 50
monitor halt
monitor sleep 50

load
continue
