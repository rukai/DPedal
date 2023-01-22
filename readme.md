# DPedal

## Build and flash firmware

Install rust via rustup then:

```bash
cargo install cargo-dfu
cd dpedal_firmware
cargo dfu --chip stm32 --release
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
* PLA
* Prusa mini with prusa slicer

### PCB

The PCB is designed to use parts that are available on JLCPCB's PCB assembly service.
The one exception is keyboard switches which you must acquire and solder yourself.

To have JLCPCB assemble the PCB for you follow these steps:

#### 1

Install kicad and its full library.
Known to work on Kicad 6.0.x

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

Select PCB Assembly.
When prompted upload the `production/bom.csv` for the bom and the `production/positions.csv` for the positions file.

## License

Licensed under the MIT license ([LICENSE.txt](license.txt))
