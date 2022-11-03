use defmt::info;
use embedded_graphics::Drawable;
use embedded_graphics::image::{ImageRaw, Image};
use embedded_graphics::prelude::Point;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use epd_waveshare::epd5in65f::{Epd5in65f, WIDTH};
use epd_waveshare::graphics::OctDisplay;
use epd_waveshare::prelude::{WaveshareDisplay, OctColor};
use rp2040_hal::spi::Disabled;
use rp_pico::pac::SPI1;

pub struct Draw<SPI, CS, BUSY, DC, RST, DELAY> {
    spi: SPI,

    epd: Epd5in65f<SPI, CS, BUSY, DC, RST, DELAY>,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Draw<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    pub fn new(
        mut spi: SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Draw<SPI, CS, BUSY, DC, RST, DELAY>, SPI::Error> {
        let epd = Epd5in65f::new(
            &mut spi, // SPI
            cs,       // CS
            busy,     // BUSY
            dc,       // DC
            rst,      // RST
            delay,    // DELAY
        )?;

        Ok(Draw { spi, epd })
    }

    pub fn draw(&mut self, img_data: &[u8], delay: &mut DELAY) -> Result<(), SPI::Error> {

        let mut display = epd_waveshare::epd5in65f::Display5in65f::default();

        self.epd.wake_up(&mut self.spi, delay)?;


        display.clear_buffer(OctColor::Black);


        let img: ImageRaw<OctColor> = ImageRaw::new(img_data, WIDTH);

        Image::new(&img, Point::zero()).draw(&mut display).unwrap();


        self.epd.update_frame(&mut self.spi, display.buffer(), delay)?;
        self.epd.display_frame(&mut self.spi, delay)?;
        delay.delay_ms(255);


        info!("Display updated!");


        self.epd.sleep(&mut self.spi, delay)?;

        Ok(())
    }
}
