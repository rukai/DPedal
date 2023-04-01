# DPedal

A large DPad you control with your foot.
It connects to your PC over usb 2.0 on a usb C port and sends user configurable HID keyboard events.
<!--It can be extended with a large array of buttons for your other foot.
But it is only designed to supplement a keyboard and/or voice dictation, not to replace them.-->

<img width="600px" alt="Image of the DPedal" src="https://media.hachyderm.io/media_attachments/files/109/760/254/454/933/863/original/559a289ef2537da9.jpeg">

## Project status

The dpad design is functional and can be built following the instructions in this repo.
The dpad has an extension port that I would like to take advantage of one day by providing a secondary unit for use by the other foot with many additional buttons.

## Usability

I use the dpedal daily, I find it most useful for simple navigation tasks:

* scrolling through large documents or pages
* tab navigation (I always use it when crossword solving)
* doom scrolling

Now that I have built two dpedal's I'll experiment with what sort of use cases dual-wielding enables and report back.

It can be quite easy to bump the left and right buttons on the dpad which can be quite disruptive for some applications, so the default configuration has the left and right buttons disabled.
They can be easily enabled when you want to try introducing them.

## Flash precompiled firmware

### Windows

1. Connect the dpedal to your PC via USB
2. Set the dpedal to flash mode by:
    1. Press and hold the FLASH button
    2. Press and release the RESET button
    3. Release the FLASH button
3. Download and run [zadig](https://zadig.akeo.ie)
    1. Options -> tick `List All Devices`
    2. In the dropdown choose `STM32 BOOTLOADER`
    3. Press `Replace Driver`
4. Download the `dpedalflash-v*-x86-64-pc-windows-msvc.zip` from [the latest release](https://github.com/rukai/DPedal/releases/latest)
5. Unzip it and navigate to the folder containing `dpedalflash.exe` and `example-config.kdl` in file explorer
6. Open `example-config.kdl` in a text editor and edit it to specify your desired keymapping (or leave it as default)
7. In file explorer press `File` -> `Open Windows Powershell`
8. In powershell write `./dpedalflash example-config.kdl` and press enter

### Other OS's

1. Download the firmware flasher for your system from [the latest release](https://github.com/rukai/DPedal/releases/latest)
2. Extract the tar or zip file
3. Modify `example-config.kdl` to specify your desired keymapping
4. Set the dpedal to flash mode by:
    1. Press and hold the FLASH button
    2. Press and release the RESET button
    3. Release the FLASH button
5. Navigate your terminal to the extracted folder and run `./dpedalflash example-config.kdl`

## Compile and flash firmware

1. Set the dpedal to flash mode by:
    1. Press and hold the FLASH button
    2. Press and release the RESET button
    3. Release the FLASH button
2. If you are on windows you will now need to follow the zadig instructions from the precompiled firmware section
3. Install rust via rustup then:

```bash
git clone https://github.com/rukai/DPedal
cd dpedal/dpedal_flash
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

For some prints I change the layer height for faster print times, but feel free to ignore these and stick with 0.10mm everywhere:

* Footrest:
  * 0.10mm layer height
  * 15% support cubic infill
  * height range modifiers (needs the advanced UI)
    * layers: 0-10mm -> height: 0.15mm
    * layers: 10-20mm -> height: 0.20mm
* angled joiner:
  * 0.15mm layer height (QUALITY profile)
* switch plate:
  * 0.20mm layer height (QUALITY profile)
* base:
  * 0.10mm layer height (QUALITY profile)
  * height range modifiers (needs the advanced UI)
    * layers: 0-3mm -> height: 0.20mm

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
