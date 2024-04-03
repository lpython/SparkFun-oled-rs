#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{
    prelude::*,
    pixelcolor::BinaryColor,
    primitives::{Circle, Rectangle, Triangle, PrimitiveStyleBuilder},
    image::{Image, ImageRaw},
};
use panic_halt as _;
use ssd1306::{prelude::*, Ssd1306,I2CDisplayInterface, size::DisplaySize64x48, command::Command};
use ssd1306::mode::BufferedGraphicsMode;

use stm32f1xx_hal::{
    timer::Timer,
    i2c::{BlockingI2c, DutyCycle, Mode},
    prelude::*,
    stm32,
};
use nb::block;

mod rust_logo;

const WIDTH: u8 = 64;
const HEIGHT: u8 = 48;
type SparkFunDisplay<DI> = Ssd1306<DI, DisplaySize64x48, BufferedGraphicsMode<DisplaySize64x48>>;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();

    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

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

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();


    let interface = I2CDisplayInterface::new_alternate_address(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize64x48, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    // let test = (0x20, 0x00);
    display.set_draw_area( (0x20u8, 0x00u8), ( 0x20u8 + 64, 0x06u8)).unwrap();
    // display.set_column(0).unwrap();
    // display.set_row(0).unwrap();
    display.draw(rust_logo::IMAGE).unwrap();
    // display.set_column_2(0).unwrap();
    // display.set_row(0).unwrap();    
    // display.bounded_draw(rust_logo::IMAGE, 64, (0,0), (64, 48)).unwrap();

    // let raw: ImageRaw<BinaryColor> = ImageRaw::new(rust_logo::IMAGE, 64);

    // let im = Image::new(&raw, Point::new(32, 0));c

    // im.draw(&mut display).unwrap();

    // display.flush().unwrap();

    use embedded_graphics::{prelude::*, primitives::Rectangle};

    let rect = Rectangle::new(Point::new(10, 20), Size::new(3, 4));

    assert_eq!(rect.columns(), 10..13);

    simple_example(&mut display);

    loop {

        block!(timer.wait()).unwrap();
        led.set_high();
        block!(timer.wait()).unwrap();
        led.set_low();
    }
}

fn simple_example<DI>( display: &mut SparkFunDisplay<DI>) 
where
    DI: WriteOnlyDataCommand
{

    let yoffset = 4;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::On)
        .build();

    // screen outline
    // default display size is 128x64 if you don't pass a _DisplaySize_
    // enum to the _Builder_ struct
    Rectangle::new(Point::new(8, 8), Size::new( 4u32,  10u32))
        .into_styled(style)
        .draw(display)
        .unwrap();

    // // triangle
    // Triangle::new(
    //     Point::new(16, 16 + yoffset),
    //     Point::new(16 + 16, 16 + yoffset),
    //     Point::new(16 + 8, yoffset),
    // )
    // .into_styled(style)
    // .draw(display)
    // .unwrap();

    // // square
    // Rectangle::new(Point::new(1, yoffset), Size::new_equal(16))
    //     .into_styled(style)
    //     .draw(display)
    //     .unwrap();

    // // circle
    // Circle::new(Point::new(10, yoffset), 16)
    //     .into_styled(style)
    //     .draw(display)
    //     .unwrap();

    display.flush().unwrap();
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}