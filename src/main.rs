#![no_main]
#![no_std]

#[allow(unused_imports)]
#[allow(unused_extern_crates)] //  bug rust-lang/rust#53964
extern crate panic_itm; // panic handler

use core::fmt::{self, Debug};

use cortex_m::{asm, iprintln};
use cortex_m_rt::entry;
use cortex_m::peripheral::ITM;
use m::Float;
use stm32f3xx_hal::{
    delay::Delay, 
    gpio::{Alternate, Gpiob, OpenDrain, Pin, U}, 
    i2c::I2c, 
    pac::{self, I2C1}, 
    prelude::*
};
use embedded_hal::{
    delay::DelayNs,
    i2c::{self, Operation, SevenBitAddress},
};
use switch_hal::OutputSwitch;
use switch_hal::ToggleableOutputSwitch;

use lsm303agr::Lsm303agr;

mod leds;

// Lsm303agr crate expects a nano-second delay, but stm32f3xx-hal crate provides a micro-second delay
struct MyDelay(Delay);

impl DelayNs for MyDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.0.delay_us(ns / 1_000);
    }
}

type I2cBus = I2c<I2C1, (Pin<Gpiob, U<6>, Alternate<OpenDrain, 4>>, Pin<Gpiob, U<7>, Alternate<OpenDrain, 4>>)>;
/// For connection lsm303agr crate to stm32f3xx-hal crate 
struct MyI2C(I2cBus);

impl Debug for MyI2C {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyI2C()")
    }
}

/// LED compass direction as noted on the board
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Direction
{
    North = 0,
    NNE = 1,
    NorthEast = 2,
    NEE = 3,
    East = 4,
    SEE = 5,
    SouthEast = 6,
    SSE = 7,
    South = 8,
    SSW = 9,
    SouthWest = 10,
    SWW = 11,
    West = 12,
    NWW = 13,
    NorthWest = 14,
    NNW = 15
}

impl Direction {
    fn from_angle(angle: i32) -> Direction {
        let angle = normalize(angle);
        match angle {
            0..=11 => Direction::North,
            12..=33 => Direction::NNE,
            34..=56 => Direction::NorthEast,
            57..=78 => Direction::NEE,
            79..=101 => Direction::East,
            102..=123 => Direction::SEE,
            124..=146 => Direction::SouthEast,
            147..=168 => Direction::SSE,
            169..=191 => Direction::South,
            192..=213 => Direction::SSW,
            214..=236 => Direction::SouthWest,
            237..=258 => Direction::SWW,
            259..=281 => Direction::West,
            282..=303 => Direction::NWW,
            304..=326 => Direction::NorthWest,
            327..=348 => Direction::NNW,
            _ => Direction::North
        }
    }
}

fn normalize(angle: i32) -> i32 {
    let mut new_angle = angle;
    while new_angle < 0 {
        new_angle += 360;
    }
    while new_angle >= 360 {
        new_angle -= 360;
    }
    new_angle
}

#[entry]
fn main() -> ! {
    let (i2c1, delay, mut itm, mut leds) =  init_cm();

    let _ = leds[0].on();
    iprintln!(
        &mut itm.stim[0],
        "Mag - ITM start", 
    );

    let mut my_delay = MyDelay(delay);
    let my_i2c = MyI2C(i2c1);
    let _ = leds[1].on();  

    let mut sensor = Lsm303agr::new_with_i2c(my_i2c);
    let _ = leds[2].on();   
    
    sensor.init().unwrap();
    let _ = leds[3].on();   
    let mut mag_sensor = sensor.into_mag_continuous().unwrap();
    let _ = leds[4].on();  
    asm::delay(8_000_000);

    for led in leds.iter_mut() {
        let _ = led.off();
    }

    loop {
        let status = mag_sensor.magnetic_field();
        if let Ok(status) = status {
            let (x, y, z) = (status.x_nt(), status.y_nt(), status.z_nt());
            iprintln!(
                &mut itm.stim[0], 
                "Magnetic Field: x {} y {} z {}", x, y, z);
            let theta = (y as f32).atan2(x as f32);
            let deg = (theta * 180.0 / core::f32::consts::PI) as i32;
            iprintln!(
                &mut itm.stim[0], 
                "Angle: rad {} deg {}", theta, deg);
            let dir = Direction::from_angle(deg); 
            iprintln!(
                &mut itm.stim[0], 
                "Direction: {:?}", dir);
            for led in leds.iter_mut() {
                let _ = led.off();
            }
            activate_leds(&mut leds, dir);        
        } else {
            for i in (0..6).step_by(2) {
                let _ = leds[i].toggle();
                let _ = leds[i + 1].toggle();
            }
        }

        my_delay.delay_ms(500_u32);
    }
}

fn activate_leds(leds: &mut [leds::Led; 8], dir: Direction) {
    match dir {
        Direction::North => {
            let _ = leds[0].on();
        }
        Direction::NNE => {
            let _ = leds[0].on();
            let _ = leds[1].on();
        }
        Direction::NorthEast => {
            let _ = leds[1].on();
        }
        Direction::NEE => {
            let _ = leds[1].on();
            let _ = leds[2].on();
        }
        Direction::East => {
            let _ = leds[2].on();
        }
        Direction::SEE => {
            let _ = leds[2].on();
            let _ = leds[3].on();
        }
        Direction::SouthEast => {
            let _ = leds[3].on();
        }
        Direction::SSE => {
            let _ = leds[3].on();
            let _ = leds[4].on();
        }
        Direction::South => {
            let _ = leds[4].on();
        }
        Direction::SSW => {
            let _ = leds[4].on();
            let _ = leds[5].on();
        }
        Direction::SouthWest => {
            let _ = leds[5].on();
        }
        Direction::SWW => {
            let _ = leds[5].on();
            let _ = leds[6].on();
        }
        Direction::West => {
            let _ = leds[6].on();
        }
        Direction::NWW => {
            let _ = leds[6].on();
            let _ = leds[7].on();
        }
        Direction::NorthWest => {
            let _ = leds[7].on();
        }
        Direction::NNW => {
            let _ = leds[7].on();
            let _ = leds[0].on();
        }
    }
} 

pub fn init_cm() -> (I2cBus, Delay, ITM, [leds::Led; 8]) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let leds = crate::leds::Leds::new(
        gpioe.pe8,
        gpioe.pe9,
        gpioe.pe10,
        gpioe.pe11,
        gpioe.pe12,
        gpioe.pe13,
        gpioe.pe14,
        gpioe.pe15,
        &mut gpioe.moder,
        &mut gpioe.otyper,
    );

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper,&mut gpiob.afrl);
    let sda = gpiob.pb7.into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper,&mut gpiob.afrl);

    let i2c = I2c::new(dp.I2C1, (scl, sda), 400_000.Hz(), clocks, &mut rcc.apb1);

    let delay = Delay::new(cp.SYST, clocks);

    (i2c, delay, cp.ITM, leds.into_array()) 
}


/// For connectioning lsm303agr crate to stm32f3xx-hal crate
#[derive(Debug)]
struct MyError(stm32f3xx_hal::i2c::Error);

impl  i2c::Error for MyError {
    fn kind(&self) -> i2c::ErrorKind {
        match self.0 {
            stm32f3xx_hal::i2c::Error::Arbitration => i2c::ErrorKind::ArbitrationLoss,
            stm32f3xx_hal::i2c::Error::Bus => i2c::ErrorKind::Bus,
            stm32f3xx_hal::i2c::Error::Nack => i2c::ErrorKind::NoAcknowledge(i2c::NoAcknowledgeSource::Unknown),
            _ => i2c::ErrorKind::Other
        }
    }
}


impl  i2c::ErrorType for MyI2C {
    type Error = MyError;
}

impl i2c::I2c<SevenBitAddress> for MyI2C {
    fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>]
    ) -> Result<(), Self::Error> 
    {
        for operation in operations {
            match operation {
                Operation::Write(write) => {
                    self.0.write(address, write).map_err(MyError)?
                }
                Operation::Read(read) => {
                    self.0.read(address, read).map_err(MyError)?
                }
            }
        }
        Ok(())
    }
}