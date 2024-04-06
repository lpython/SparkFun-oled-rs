#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use panic_halt as _;

use stm32f1xx_hal::{
    timer::Timer,
    i2c::{BlockingI2c, DutyCycle, Mode},
    prelude::*,
    stm32,
};
use nb::block;

use ssd1306::{prelude::*, Ssd1306,I2CDisplayInterface, size::DisplaySize64x48 };
use ssd1306::mode::BufferedGraphicsMode;

use embedded_graphics::{
    prelude::*,
    pixelcolor::BinaryColor,
    image::{Image, ImageRaw}
};


mod rust_logo;

const WIDTH: i32 = 64;
const HEIGHT: u8 = 48;
const CENTER_X: i32 = WIDTH / 2;
// const CENTER_Y: u8 = HEIGHT / 2;   

const GAUGE_CENTER: (f32, f32) = (CENTER_X as f32, (HEIGHT as f32 * 0.75));
const GAUGE_WIDTH: f32 = 3.0;
const LINE_LENGTH: f32 = 20.0;

type SparkFunDisplay<DI> = Ssd1306<DI, DisplaySize64x48, BufferedGraphicsMode<DisplaySize64x48>>;
#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();

    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain();

    let mut gpiob = dp.GPIOB.split();

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 100_000.Hz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        1000,
        10,
        1000,
        1000,
    );

    let mut gpioc = dp.GPIOC.split();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(6.Hz()).unwrap();

    let interface = I2CDisplayInterface::new_alternate_address(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize64x48, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    display.set_draw_area( (0x20u8, 0x00u8), ( 0x20u8 + 64, 0x06u8)).unwrap();
   
    const IMAGE_WIDTH : i32 = 48;
    const IMAGE_CENTER : i32 = IMAGE_WIDTH /2;
  
    let raw: ImageRaw<BinaryColor> = ImageRaw::new(rust_logo::IMAGE, IMAGE_WIDTH as u32);

    let im = Image::new(&raw, Point::new(CENTER_X - IMAGE_CENTER, 0));

    im.draw(&mut display).unwrap();

    display.flush().unwrap();


    loop {
        
    }
}


#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}