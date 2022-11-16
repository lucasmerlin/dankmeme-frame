use cyw43::NetDevice;
use defmt::{info, warn};
use embassy_net::{Stack, Ipv4Address};
use embassy_net::tcp::TcpSocket;
use embassy_rp::gpio::Input;
use embassy_rp::peripherals::{PIN_15, PIN_17, PIN_2};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::prelude::Point;
use embedded_graphics::Drawable;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Write as SpiWrite;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_io::asynch::Write;
use epd_waveshare::epd5in65f::{Display5in65f, Epd5in65f, WIDTH};
use epd_waveshare::graphics::OctDisplay;
use epd_waveshare::prelude::{OctColor, WaveshareDisplay};
use embassy_futures::select::{select, select3, Either3, select4, Either4};
use arrform::arrform;
use arrform::ArrForm;

use crate::dns::dns_request;

const MALMAL: &str = "malmal";
const MEMES: &str = "memes";

pub struct Draw<'a, SPI, CS, BUSY, DC, RST> {
    spi: SPI,

    epd: Epd5in65f<SPI, CS, BUSY, DC, RST, Delay>,

    display: Display5in65f,

    delay: Delay,

    refresh_button: Input<'a, PIN_15>,
    switch_button: Input<'a, PIN_17>,
    clear_button: Input<'a, PIN_2>,
}

impl<'a, SPI, CS, BUSY, DC, RST> Draw<'a, SPI, CS, BUSY, DC, RST>
where
    SPI: SpiWrite<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    pub fn new(
        mut spi: SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,

        refresh_button: Input<'a, PIN_15>,
        switch_button: Input<'a, PIN_17>,
        clear_button: Input<'a, PIN_2>,
    ) -> Result<Draw<'a, SPI, CS, BUSY, DC, RST>, SPI::Error> {

        let mut delay = Delay;

        let epd = Epd5in65f::new(
            &mut spi, // SPI
            cs,       // CS
            busy,     // BUSY
            dc,       // DC
            rst,      // RST
            &mut delay,    // DELAY
        )?;

        Ok(Draw {
            spi,
            epd,
            display: Display5in65f::default(),
            delay,
            refresh_button,
            switch_button,
            clear_button,
        })
    }

    pub fn draw(&mut self) -> Result<(), SPI::Error> {

        self.epd.wake_up(&mut self.spi, &mut self.delay)?;

        self.epd
            .update_frame(&mut self.spi, self.display.buffer(), &mut self.delay)?;
        self.epd.display_frame(&mut self.spi, &mut self.delay)?;
        self.delay.delay_ms(1000u16);

        info!("Display updated!");

        self.epd.sleep(&mut self.spi, &mut self.delay)?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), SPI::Error> {
        self.epd.wake_up(&mut self.spi, &mut self.delay)?;

        self.epd
        .clear_frame(&mut self.spi, &mut self.delay)?;
        self.delay.delay_ms(1000u16);

        info!("Display cleared!");

        self.epd.sleep(&mut self.spi, &mut self.delay)?;

        Ok(())
    }

    pub async fn run<'b>(&mut self, stack: &Stack<NetDevice<'b>>) {
        // And now we can use it!

        let mut rx_buffer = [0; 4096];
        let mut tx_buffer = [0; 4096];
        let mut buf = [0; 4096];

        let mut path = MALMAL;

        loop {


            let addr = dns_request(stack, "dankmeme-frame.fly.dev.").await.unwrap();

            info!("Got dns address: {}", addr);


            info!("Making http request...");

            let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
            socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

            info!("Connecting to 1.1.1.1....");
            if let Err(e) = socket
                .connect((addr, 80))
                .await
            {
                warn!("error: {:?}", e);

                Timer::after(Duration::from_millis(5000)).await;
                continue;
            }

            info!("Received connection from {:?}", socket.remote_endpoint());

            let af = arrform!(512, "GET /{} HTTP/1.0\r\nHost: dankmeme-gallery.onrender.com\r\n\r\n", path);

            socket.write_all(af.as_bytes()).await.unwrap();

            let display_buf = self.display.get_mut_buffer();
            let mut offset = 0;
            let mut header = true;

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => {
                        warn!("read EOF");
                        break;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        warn!("read error: {:?}", e);
                        break;
                    }
                };

                let body_offset = if header {
                    let start_pos = buf.windows(4).position(|window| {
                        window == b"\r\n\r\n"
                    }).expect("Couldn't find end of http header!") + 4;
                    let string = core::str::from_utf8(&buf[..start_pos]).unwrap();
                    header = false;
                    info!("Header: {}", string);
                    start_pos
                } else {
                    0
                };

                let source = &buf[body_offset..n];

                let len = source.len();

                let offset_end = usize::min(offset + len, display_buf.len() - 1);

                let target = &mut display_buf[offset..offset_end];

                if target.len() != source.len() {
                    warn!("Target len({}) didn't match source len({})!", target.len(), source.len());
                    break;
                }

                target.copy_from_slice(source);
                offset += len;

                info!("read {} bytes", n);
            }

            info!("Drawing...");

            self.draw();

            info!("Finished....");

            let refresh_timeout = Timer::after(Duration::from_millis(60 * 20 * 1000));
            let refresh_button = self.refresh_button.wait_for_rising_edge();
            let switch_button = self.switch_button.wait_for_rising_edge();
            let clear_button = self.clear_button.wait_for_rising_edge();

            let event = select4(refresh_timeout, refresh_button, switch_button, clear_button).await;

            match event {
                Either4::Third(_) => {
                    match path {
                        MALMAL => path = MEMES,
                        MEMES => path = MALMAL,
                        _ => path = MALMAL,
                    }
                }
                Either4::Fourth(_) => {
                    self.clear().ok();
                    select(self.refresh_button.wait_for_rising_edge(), Timer::after(Duration::from_secs(60 * 20))).await;
                }
                _ => ()
            }
        }
    }
}
