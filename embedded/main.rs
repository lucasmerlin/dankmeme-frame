#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::{Dimensions, Point, Primitive, Size},
    primitives::{
        Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text, TextStyleBuilder},
    Drawable, image::{Image, ImageRaw},
};
use embedded_hal::digital::v2::OutputPin;
use epd_waveshare::{
    epd5in65f::{HEIGHT, WIDTH},
    graphics::{DisplayRotation, OctDisplay},
    prelude::{Color, OctColor, WaveshareDisplay},
};
use fugit::RateExtU32;
use panic_probe as _;
use rp2040_hal as hal;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use tinybmp::Bmp;
use tinytga::Tga;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // SPI declaration
    let _spi_sclk = pins.gpio10.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio11.into_mode::<hal::gpio::FunctionSpi>();
    let spi = hal::spi::Spi::<_, _, 8>::new(pac.SPI1);

    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        // you can put cookie (increase the speed) in it but I don't recommend it.
        4_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );
    // End of SPI declaration

    // Start the rest of pins needed to communicate with the screen
    let mut cs = pins.gpio9.into_push_pull_output(); // CS
    cs.set_high().unwrap();
    let busy = pins.gpio13.into_pull_up_input(); // BUSY
    let dc = pins.gpio8.into_push_pull_output(); // DC
    let rst = pins.gpio12.into_push_pull_output(); // RST

    let mut epd = epd_waveshare::epd5in65f::Epd5in65f::new(
        &mut spi,   // SPI
        cs,         // CS
        busy,       // BUSY
        dc,         // DC
        rst,        // RST
        &mut delay, // DELAY
    )
    .expect("Display init failed");

    let mut display = epd_waveshare::epd5in65f::Display5in65f::default();

    epd.wake_up(&mut spi, &mut delay).unwrap();


    display.clear_buffer(OctColor::Black);



//    let tga = Tga::from_slice(include_bytes!(concat!(
//            env!("CARGO_MANIFEST_DIR"),
//    "/meme.tga"
//    )))
//    .unwrap();
//
//    let image: Image<Tga<OctColor>> = Image::new(&tga, Point::new(0, 0));
//    image.draw(&mut display).unwrap();

//    let bmp = Bmp::<Rgba888>::from_slice(include_bytes!("../meme.bmp")).unwrap();
//    Image::new(&bmp, Point::new(0, 0)).draw(&mut display);

    let img_data = include_bytes!("../../dankmeme-gallery/image.bin");

//    let img: ImageRaw<OctColor> = ImageRaw::new(img_data, WIDTH);
//
//    Image::new(&img, Point::zero()).draw(&mut display).unwrap();


    epd.update_frame(&mut spi, img_data, &mut delay)
        .unwrap();
    epd.display_frame(&mut spi, &mut delay).unwrap();
    delay.delay_ms(1000);


    info!("Display updated!");


    epd.sleep(&mut spi, &mut delay).unwrap();

    let mut led_pin = pins.led.into_push_pull_output();

    // If the led blink, everything it's ok (big debug skills)
    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}

// End of file
