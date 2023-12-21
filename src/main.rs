#![no_std]
#![no_main]

extern crate panic_halt;

use esp32_hal::{
    clock_control::{sleep, ClockControl, XTAL_FREQUENCY_AUTO},
    dport::Split,
    embedded_flash::Flash,
    prelude::*,
    spi::{self, SPI},
    target,
    wifi::WIFI,
};
use esp32_hal::wifi::*;
use tiny_http::{Method, Request, Response, Server};

#[entry]
fn main() -> ! {
    // Initialize peripherals
    let dp = target::Peripherals::take().unwrap();
    let (_, dport_clock) = dp.DPORT.split();

    let clock_control = ClockControl::new(dp.RTCCNTL, dp.APB_CTRL)
        .set_xtal_frequency(XTAL_FREQUENCY_AUTO)
        .enable_pll();

    // Set up WiFi
    let wifi = WIFI.lock(&clock_control, dport_clock);
    wifi.start_ap("ESP32-Rust", "password").unwrap();

    // Set up SPI
    let mut spi_bus = SPI::new(
        dp.SPI3,
        spi::Pins {
            sck: gpio::gpiob::PB3,
            mosi: gpio::gpiob::PB4,
            miso: gpio::gpiob::PB5,
        },
        spi::MODE_0,
        10_000_000.hz(),
        clock_control.peripheral_clock(),
    );

    // Set up embedded flash
    let flash = Flash::new(dp.EXTMEM);

    // Set up HTTP server
    let server = Server::http("0.0.0.0:80").unwrap();

    // Main loop
    loop {
        // Read from SPI flash
        let data_read = read_spi_flash(&mut spi_bus, &flash, 0x1000, 256);

        // Write to SPI flash
        let data_to_write = [0x01, 0x02, 0x03];
        write_spi_flash(&mut spi_bus, &flash, 0x2000, &data_to_write);

        // Delete from SPI flash (erase sector as needed)
        delete_spi_flash_sector(&mut spi_bus, &flash, 0x3000);

        // Verify data in SPI flash
        let data_to_verify = [0x01, 0x02, 0x03];
        let is_verified = verify_spi_flash(&mut spi_bus, &flash, 0x2000, &data_to_verify);

        // Handle incoming HTTP requests
        if let Ok(request) = server.recv() {
            // Process the request
            handle_request(request);
        }

        // Your server logic goes here

        // Sleep for a while
        sleep(1.seconds());
    }
}

fn read_spi_flash(spi: &mut SPI, flash: &Flash, address: usize, length: usize) -> Vec<u8> {
    // Read data from SPI flash
    let mut data_read = vec![0; length];
    flash.read(spi, address, &mut data_read).unwrap();
    data_read
}

fn write_spi_flash(spi: &mut SPI, flash: &Flash, address: usize, data: &[u8]) {
    // Write data to SPI flash
    flash.write_enable(spi).unwrap();
    flash.write(spi, address, data).unwrap();
}

fn delete_spi_flash_sector(spi: &mut SPI, flash: &Flash, address: usize) {
    // Erase the sector in SPI flash
    flash.erase_sector(spi, address).unwrap();
}

fn verify_spi_flash(spi: &mut SPI, flash: &Flash, address: usize, data: &[u8]) -> bool {
    // Read data from SPI flash and compare it with the provided data
    let mut data_read = vec![0; data.len()];
    flash.read(spi, address, &mut data_read).unwrap();
    data_read == data
}

fn handle_request(request: Request) {
    // Respond with a simple working text
    let response = Response::from_string("ESP32 Web Server is working!");
    request.respond(response).unwrap();
}
