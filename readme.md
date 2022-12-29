# DPedal

## Manufacturing

The 3D printed plastic parts are available here, slice and print them yourself.

The PCB is designed to use parts that are available on JLCPCB's PCB assembly service.
The one exception is keyboard switches which you must acquire and solder yourself.

To have JLCPCB assemble the PCB for you follow these steps:

### 1
Install kicad and its full library.
Known to work on Kicad 6.0.x

### 2
Open kicad 
Select "Plugin and Content Manager" and install the following plugins:
* Fabrication Toolkit - This is a JLCPCB specific plugin for generating the fabrication files that JLCPCB accept
* Keyswitch Kicad Library - This adds keyboard switch footprints

### 3
Open the dpedal kicad project, open the PCB editor, then press the Fabrication Toolkit icon.
This will generate the files you need in a `production/` folder.

### 4
Run through the JLCPCB PCB assembly ordering process using the generated files.

## Build and flash firmware

TODO - firmware doesnt exist yet
