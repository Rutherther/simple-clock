# Simple clock
This is just a small project that aims to make a simple clock, using STM32F103C8T6.
The clock will be shown on a seven segment display. There will be multiple modes to show
on the clock display.

## HW
STM32F103C8T6 is used as the _brains_. It has RTC that may be powered by CR2032 battery.
It's connected to a USB header, allowing to communicate with a computer. The chip is capable
of being a device, only, not host.
There is a header with 3V3, SWDIO, SWCLK, GND to allow programming the chip using a tool such as STLink.
ASM1117-3.3 is used for stepping down from 5V to 3V3.
3V3 should not be connected whilst USB is connected as well. This would short output of the LDO to 3V3.
There is no protection!

There are 8 seven segment displays,
4 of them are blue, 4 of them are yellow, the whole display looks like this: Y.Y.BB:BBY.Y.
(Y for yellow digit, B for blue digit)
There are 4 LEDs and 4 buttons, the LEDs are next to the buttons, making each LED "associated" with a button.
That makes it possible to light up an LED next to button, if the button is pressed etc.

The digits of the seven segment display are connected to PNP transistors.
When pulling the pin connected to the transistor low, the digit is turned on.
The pins used may be connected to timers, and PWM may be utilized to set
brightness.
Segments get turned on by being low as well.

Mapping of pins to peripherals is documented in [pins](docs/pins.md).

## Features / Roadmap

- [x] Remember date, time
  - [x] Time
  - [x] Date
- [x] Set time and date using simple interface
- [x] Switch between view modes by button
- [x] Show time, date
- [x] Adjust brightness using PWM
- [x] Auto adjust brightness based on time
- [ ] Stopwatch
- [ ] USB communication
  - [ ] Set time, Get time
  - [ ] Show arbitrary text or number on the display
  - [ ] Start stopwatch
  
## Usage
The clock has two modes, default mode, and edit mode.
Default mode displays the current time, edit mode is
for editing the current time.

### Default mode
This is entered upon reset. Upon startup,
it displays only current time as hours and minutes.

First button may change the current view, there are
four views.
1. time - hours and minutes only
2. time - with seconds
3. time and date - hours, minutes, day in month and month
4. date - year, day in month and month

The second button switches to edit mode.

Third and fourth buttons change brightness.
The brightness is normally automatically adjusted
based on the current time set on the clock.
It may be temporarily changed using these buttons.
The change will last 30 minutes.
Third button decreases brightness, fourth increases it.

### Edit mode
In the edit mode, it's possible to change the time and date.
Upon entering edit mode, hours will be selected to be changed.

The currently selected field will blink. To select the next field,
the first button should be used. The order of the fields edited is:
hours, minutes, seconds, year, month, day.

To increment or decrement the current field, second and third
buttons should be used, respectively.

To save the time and date, fourth button does the job.
It may be pressed when editing any field.

When in the edit mode, the time is paused. If you enter the
edit mode by accident, the clock will probably get behind,
because a couple of seconds will pass before you exit the edit mode.

## Images of the clock
### Front, off
<img src="img/front_off.jpg" alt="Front, off" width=800>

### Back
<img src="img/back.jpg" alt="Back" width=800>

### Front, with seconds
<img src="img/front_seconds.jpg" alt="Front, with seconds" width=800>

### Front, with clock and date
<img src="img/front_clock_date.jpg" alt="Front, clock and date" width=800>

## What's next
Unfortunately, I did not solder the USB correctly on my board, so I cannot
currently test sending and receiving data using the USB, that is the reason
why the current program does not even support USB. In the future, the firmware
should support USB communication and a program for a computer should be made,
with some kind of CLI for communication with the board.
