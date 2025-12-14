# DPedal

A large foot controlled DPad.
It is built around a raspberry pi pico and connects to your PC over usb to send user configurable HID keyboard and mouse events.

It consists entirely of 3d printed parts except for the PCB, keyboard switches and 2 side button caps.
You can build your own from this design if you are comfortable with 3d printing, ordering assembled PCB's from a manufacturer and a tiny bit of soldering.

To build your own dpedal follow [the instructions here](https://dpedal.com/build.html).

This repo contains the source for:

* The [firmware](https://github.com/rukai/DPedal/tree/main/dpedal_firmware)
* The [flashing tool](https://github.com/rukai/DPedal/tree/main/dpedal_flash)
* The [PCB](https://github.com/rukai/DPedal/tree/main/pcb)

The cad parts are not in this repo, instead they are in [onshape](https://cad.onshape.com/documents/3322725aad79769314b0a0dc/w/7eb6c11c4e7989e30d759821/e/7eadfca9ff0dbd31823e3a21).

## Changes from [DPedal v2](https://github.com/rukai/dpedal_legacy)

* smaller height
  * foot is less raised off the ground
  * easier to take with you
* make buttons recessed into shell, protect them from snapping off
* 2 buttons instead of 4, I rarely use the buttons, I think having less buttons will make it easier to hit the right one.
* web-based flashing tool
