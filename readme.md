# DPedal

A large DPad you control with your foot.
It connects to your PC over usb 2.0 on a usb C port and sends user configurable HID keyboard events.
<!--It can be extended with a large array of buttons for your other foot.
But it is only designed to supplement a keyboard and/or voice dictation, not to replace them.-->

<img width="600px" alt="Image of the DPedal" src="https://media.hachyderm.io/media_attachments/files/109/760/254/454/933/863/original/559a289ef2537da9.jpeg">

## Project status

An extension is being worked on to provide many additional buttons to the DPedal, but the basic dpad design is functional.

## Flash precompiled firmware

Windows does not work yet.
Macos is untested.

1. Download the firmware flasher for your system from [the latest release](https://github.com/rukai/DPedal/releases/latest)
2. Extract the tar or zip file
3. Modify example-config.kdl to specify your desired keymapping
4. Navigate your terminal to the extracted folder and run `./dpedalflash example-config.kdl`

## Compile and flash firmware

Install rust via rustup then:

```bash
cd dpedal_flash
cargo run --release -- example-config.kdl
```

## Manufacturing

### 3D Printing

The 3D printed plastic parts are available [here](https://cad.onshape.com/documents/b3650977a607511c32026f52/w/79027c5ddd8ad99ee7db1e2a/e/7192077cb58abe7f31bd20c3?renderMode=0&uiState=63ad8d5084623c01cce27891).
Convert each part to an STL by going through the `pad`, `base` and `switch plate` tabs at the bottom of the page.
For each tab, right click the tab and select export.
Set the format to STL and keep the rest as default.
Then slice and print each STL yourself.

My configuration:

* Every part is printed in its default orientation with no supports
* 0.10mm layer height
* 15% gyroid infill (prusa slicer's default)
* PLA
* Prusa mini with prusa slicer

For the footrest I deviated from this to keep the print time manageable:

* 0.10mm layer height
* 15% support cubic infill
* height range modifiers (needs the advanced UI)
  * layers: 0-10mm -> height: 0.15mm
  * layers: 10-20mm -> height: 0.20mm

### PCB

The PCB is designed to use parts that are available on JLCPCB's PCB assembly service.
The one exception is [kailh switch hot swap sockets](https://www.aliexpress.com/item/32959301642.html) which you must acquire and solder yourself.

To have JLCPCB assemble the PCB for you follow these steps:

#### 1

Install kicad and its full library.
Known to work on Kicad 7.0.x

#### 2

Open kicad.
Select "Plugin and Content Manager" and install the following plugins:

* Fabrication Toolkit - This is a JLCPCB specific plugin for generating the fabrication files that JLCPCB accept
* Keyswitch Kicad Library - This adds keyboard switch footprints

#### 3

Open the dpedal kicad project, open the PCB editor, then press the Fabrication Toolkit icon.
This will generate the files you need in a `production/` folder.

#### 4

Run through the JLCPCB PCB assembly ordering process using the generated files.
<https://cart.jlcpcb.com/quote>

Add a gerber file: `production/gerber.zip`

The default options should all be fine but I specify:

* Surface Finish: LeadFree HASL
* PCB Color: Go wild!
* Remove Order Number - Specify a location

Enable PCB Assembly and set:

* "Confirm Parts Placement" to yes. (the assembly preview they show has a few parts in the wrong position so I believe this option is needed for jlcpcb to manually fix the part placement on their end)

When prompted upload the `production/bom.csv` for the BOM file and the `production/positions.csv` for the CPL file.

About a day after purchasing you will need to give confirmation that the parts are correctly placed on the PCB.
Check your orders on the website and click on "DFM Analysis"
There should be an "Original Part Placement" which is completely broken.
And the "Corrected Part Placment" which should look like this:
![correctedPartPlacement.png]()

### Cherry MX compatible switches

You will need to obtain your own cherry MX switches.
I use gateron green switches because foot activated inputs benefit from a lot of extra activation force but feel free to experiment yourself.

## License

Licensed under the MIT license ([LICENSE.txt](license.txt))
