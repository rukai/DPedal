# DPedal

A large foot controlled DPad.
It connects to your PC over usb 2.0 on a usb C port and sends user configurable HID keyboard events.

It consists entirely of 3d printed parts except for the PCB and keyboard switches.
You can build your own from this design if you are comfortable with 3d printing, ordering assembled PCB's from a manufacturer and a tiny bit of soldering.

## Changes from DPedal v2

* smaller height
  + foot is less raised off the ground
  + easier to take with you
* make buttons recessed into shell, protect them from snapping off
* 2 buttons instead of 4, I rarely use the buttons, I think having less buttons will make it easier to hit the right one.
* web-based flashing tool
  + port this to rust https://github.com/ArcaneNibble/i-cant-believe-its-not-webusb/blob/main/main.c
