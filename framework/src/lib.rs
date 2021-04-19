use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
};
use embedded_graphics_simulator::{SimulatorDisplay, Window};
use std::time::{Duration, Instant};

pub mod draw;
mod menu;
mod parameter;

use menu::Menu;
pub use parameter::Parameter;

pub mod prelude {
    pub use crate::{parameter::Parameter, App, AppExt};
}

const MIN_FRAME_DURATION: Duration = Duration::from_millis(1000 / 60);

pub trait App {
    type Color: PixelColor + From<BinaryColor> + Into<Rgb888>;
    const DISPLAY_SIZE: Size;

    fn new() -> Self;

    fn clear_color(&self) -> Self::Color {
        BinaryColor::Off.into()
    }

    fn menu_color(&self) -> Self::Color {
        BinaryColor::On.into()
    }

    fn parameters(&mut self) -> Vec<Parameter>;

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible>;
}

pub trait AppExt: App {
    fn run(window: Window);
}

impl<T: App> AppExt for T {
    fn run(mut window: Window) {
        let mut app = T::new();
        let mut display = SimulatorDisplay::new(T::DISPLAY_SIZE);

        let mut menu = Menu::new();
        let menu_color = app.menu_color();

        loop {
            let start = Instant::now();

            display.clear(app.clear_color()).unwrap();

            app.draw(&mut display).unwrap();

            let mut parameters = app.parameters();

            menu.draw_menu(&parameters, &mut display, menu_color)
                .unwrap();

            window.update(&display);

            if menu.handle_events(&mut parameters, &mut window) {
                break;
            }

            let frame_duration = start.elapsed();
            if frame_duration < MIN_FRAME_DURATION {
                std::thread::sleep(MIN_FRAME_DURATION - frame_duration);
            }
        }
    }
}
