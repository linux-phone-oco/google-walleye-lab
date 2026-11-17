<!-- Generated from data/ by tad. Do not edit by hand. -->

# Hardware

<a id="uart-1v8-adapter"></a>
### 1.8 V FT232-class USB-serial adapter (DSD TECH SH-U09C2 with its 1.8 V jumper)

Kind
: adapter

Status
: wanted

Fit
: claimed

Spec
: FT232 chip, logic level 1.8 V set by a jumper; an FT232R supports break, a CP2102N does not

Serves
: Pixel 2 (walleye) and other phones with UART on the USB-C SBU pins

About
: walleye-first-fastboot-boot

General
: docs:uart-1v8-adapter

Sources
: src-pixel-uart-cable,xairy-pixel-kgdb

Note
: The 1.8 V jumper is a retailer's claim: not confirmed on the Pixel 2. Check the jumper before connecting: 3.3 V can damage the phone.

Checked on
: 2026-11-17


<a id="usbc-sbu-breakout"></a>
### USB-C passthrough breakout that brings out the SBU pins

Kind
: breakout

Status
: wanted

Fit
: claimed

Spec
: USB2 data, CC1/CC2 and SBU1/SBU2 passed through (the stock Pixel 2 cable lacks the SBU and CC lines and does not work)

Serves
: Pixel 2 (walleye)

About
: walleye-first-fastboot-boot

General
: docs:usbc-sbu-breakout

Sources
: bacon-pixel-uart,src-pixel-uart-cable

Checked on
: 2026-11-17


<a id="usb-cereal"></a>
### USB-Cereal (google/usb-cereal, soldered edition)

Kind
: adapter

Status
: wanted

Fit
: claimed

Spec
: SBU1 = TX, SBU2 = RX, DIP 1.8/3.3 V, orientation-specific, not USB-C compliant; the repository is archived (2023-01-10), not an official Google product

Serves
: Pixel phones with the UART on the SBU pins

About
: walleye-first-fastboot-boot

General
: docs:usb-cereal

Price
: EUR 39.95 at soldered.com

Price on
: 2026-11-17

Sources
: google-usb-cereal,xairy-pixel-kgdb

Note
: Unconfirmed on the Pixel 2. A wrong DIP setting can damage the target; the chip edition matters (FT232R vs CP2102N).

Checked on
: 2026-11-17


<a id="uart-debug-setup"></a>
### The Pixel 2's console on UART: the breakout with the 1.8 V adapter, or USB-Cereal

Kind
: setup

Status
: wanted

Fit
: claimed

Wiring
: USB-C SBU1 (A8) is the phone's TX into the adapter's RX; SBU2 (B8) is the phone's RX from the adapter's TX; ground on A1; 1.8 V; 115200 baud. Orientation unconfirmed for walleye (the Pixel 2 and Pixel 5a are opposite, per bacon.dev). On the phone: fastboot oem uart enable (bootloader unlocked), reboot to fastboot; fastboot oem uart disable turns it off. It gives a log only.

Serves
: Pixel 2 (walleye)

About
: walleye-first-fastboot-boot

General
: docs:uart-debug-setup

Sources
: bacon-pixel-uart,src-pixel-uart-cable,xairy-pixel-kgdb,google-usb-cereal

Note
: Claimed by four sources, never tried on walleye: confirm with a run and its log (--fit-ref) before calling it confirmed.

Checked on
: 2026-11-17
