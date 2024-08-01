Getting Started
===============
Inu-Rust (Inu "Ferric" Edition) is a Rust-based firmware for the ESP32 microcontrollers. It is based on the ESP-Rust
project, which provides a Rust-based HAL for the ESP32.

Currently, this project is locked to the ESP32-S3 board, which runs the Xtensa architecture. You can read more on
getting set-up for this architecture via the [ESP-Rust documentation](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html).

There are two flavours for ESP-Rust, a `std` and `no_std` version. This project is built on the leaner `no_std` version.

Setting Up Your Environment
---------------------------
### Ubuntu Quick-Start

    sudo apt install git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libudev-dev libffi-dev libssl-dev dfu-util libusb-1.0-0
    cargo install espup espflash ldproxy cargo-generate
    espup install

You will need the `inu-cfg-flash` tool to send the device settings to the NVS partition of the device.

    git clone https://github.com/jordonsc/inu-cfg-flash
    cargo install --path inu-cfg-flash

Installing the Bootloader
-------------------------
The bootloader will be installed when you run `cargo run`, but you need to have the device powered in boot mode.

To put the device in boot mode, hold the `BOOT` button while pressing the `RESET` button. Run `cargo run --release` to
install the bootloader. When successful, it will pause with a "waiting for download" message - this is fine, reboot
the device normally and run `cargo run` again.

Compiling and Flashing Your Device
----------------------------------
From the root of the project, you can compile and flash your device with the following commands:

    # -- Boot the device in bootloader mode (hold BOOT button while pressing RESET) --

    # The first build will install the bootloader and create the correct partition table.
    cargo run --release
    
    # -- Restart the device for a normal boot --
    # The settings partition will be empty, so the device will panic when booting.

    # Flash the NVS settings partition to the device
    tools/cfg -d "inu.device"

    # When connecting again it should boot successfully
    espflash monitor  # <CTRL+R> to reboot

Thereon-after, you can use `cargo run --release` to flash the device without needing to enter bootloader mode.
