#![no_std]
#![no_main]

use core::f32::consts::PI;
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
    primitives::{Circle, Triangle, PrimitiveStyle, PrimitiveStyleBuilder},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    text::{Baseline, Text},
};

use micromath::F32Ext;

mod rust_logo;

const WIDTH: u8 = 64;
const HEIGHT: u8 = 48;
const CENTER_X: u8 = WIDTH / 2;
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

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(6.Hz()).unwrap();

    let interface = I2CDisplayInterface::new_alternate_address(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize64x48, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    // let test = (0x20, 0x00);
    display.set_draw_area( (0x20u8, 0x00u8), ( 0x20u8 + 64, 0x06u8)).unwrap();
    // display.set_column(0).unwrap();
    // display.set_row(0).unwrap();
    display.draw(rust_logo::IMAGE).unwrap();

    display.flush().unwrap();


    loop {
        for deg in 0..180 { 
            display.clear(BinaryColor::Off).unwrap();

            draw_dial(&mut display, deg as f32).unwrap();
            draw_dial_center(&mut display).unwrap();

            display.flush().unwrap();

            block!(timer.wait()).unwrap();
            led.toggle();
        }
        for deg in 180..0 { 
            display.clear(BinaryColor::Off).unwrap();

            draw_dial(&mut display, deg as f32).unwrap();
            draw_dial_center(&mut display).unwrap();
            display.flush().unwrap();

            block!(timer.wait()).unwrap();
            led.toggle();
        }
    }
}

fn write_text<DI>( display: &mut SparkFunDisplay<DI>) 
where
    DI: WriteOnlyDataCommand
{
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(display)
        .unwrap();

}

// fn draw_dial<DI>( display: &mut SparkFunDisplay<DI>, mut deg: f32) -> Result<(), SparkFunDisplay<DI>::Error>
// where
//     DI: WriteOnlyDataCommand
fn draw_dial<D>(target: &mut D, mut deg: f32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    deg += 180.0;

    let line_end_x = LINE_LENGTH * (deg * PI / 180.0).cos();
    let line_end_y = LINE_LENGTH * (deg * PI / 180.0).sin();
    let end = Point::new( (GAUGE_CENTER.0 + line_end_x) as i32, (GAUGE_CENTER.1 + line_end_y) as i32);

    let offset_deg = normalize_deg(deg + 90.0);
    let offset_x = (offset_deg * PI/180.0).cos() * GAUGE_WIDTH;
    let offset_y = (offset_deg * PI/180.0).sin() * GAUGE_WIDTH;
    let left = Point::new( (GAUGE_CENTER.0 + offset_x) as i32, (GAUGE_CENTER.1 + offset_y) as i32);

    let offset_deg = normalize_deg(deg - 90.0);
    let offset_x = (offset_deg * PI/180.0).cos() * GAUGE_WIDTH;
    let offset_y = (offset_deg * PI/180.0).sin() * GAUGE_WIDTH;
    let right = Point::new( (GAUGE_CENTER.0 + offset_x) as i32, (GAUGE_CENTER.1 + offset_y) as i32);

    Triangle::new(
        end,
        left, 
        right
    )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(target)?;
    
    Ok(())
}

fn draw_dial_center<D>(target: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let style = PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::Off)
        .stroke_color(BinaryColor::On)
        .stroke_width(1)
        .build();

    Circle::with_center(Point::new(GAUGE_CENTER.0 as i32, GAUGE_CENTER.1 as i32), (LINE_LENGTH / 2.0) as u32)
        .into_styled(style)
        .draw(target)?;
    
    Ok(())
}

 
fn normalize_deg(deg: f32) -> f32 {
    let mut deg = deg;
    while deg < 0.0 {
        deg += 360.0;
    }
    while deg > 360.0 {
        deg -= 360.0;
    }
    deg
}


#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}