#![no_std]
#![no_main]

use core::cell::RefCell;
use core::panic::PanicInfo;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::{bind_interrupts, spi};
// USB driver
use embassy_rp::peripherals::USB;
use embassy_rp::spi::{Blocking, Spi};
use embassy_rp::usb::{Driver, InterruptHandler as USBInterruptHandler};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Delay;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::text::renderer::CharacterStyle;
use log::info;
use st7789::{Orientation, ST7789};
use workshop_at_acadnet::SPIDeviceInterface;

const DISPLAY_FREQ: u32 = 64_000_000;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => USBInterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    // USB logger driver
    let driver = Driver::new(peripherals.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    info!("Raspberry Pi Started");

    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    // Display SPI pins
    let miso = peripherals.PIN_4;
    let mosi = peripherals.PIN_19;
    let clk = peripherals.PIN_18;

    // Display SPI on SPI0
    let spi_display: Spi<'_, _, Blocking> =
        Spi::new_blocking(peripherals.SPI0, clk, mosi, miso, display_config.clone());
    // SPI bus for display
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi_display));

    let display_cs = Output::new(peripherals.PIN_17, Level::High);

    // Display SPI device initialization
    let display_spi = SpiDeviceWithConfig::new(&spi_bus, display_cs, display_config);

    // Other display pins
    let rst = peripherals.PIN_0;
    let dc = peripherals.PIN_16;
    let dc = Output::new(dc, Level::Low);
    let rst = Output::new(rst, Level::Low);
    let di = SPIDeviceInterface::new(display_spi, dc);

    // Init ST7789 LCD
    let mut display = ST7789::new(di, rst, 240, 240);
    display.init(&mut Delay).unwrap();
    display.set_orientation(Orientation::Portrait).unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    // Define style
    let mut style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    style.set_background_color(Some(Rgb565::BLACK));

    // TODO write something to the display
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
