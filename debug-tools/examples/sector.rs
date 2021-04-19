use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Sector},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::{draw, prelude::*};

struct SectorDebug {
    center: Point,
    diameter: u32,
    angle_start: i32,
    angle_sweep: i32,
    stroke_width: u32,
}

impl App for SectorDebug {
    type Color = Rgb888;
    const DISPLAY_SIZE: Size = Size::new(256, 256);

    fn new() -> Self {
        Self {
            center: Point::new(128, 128),
            diameter: 50,
            angle_start: 0,
            angle_sweep: 30,
            stroke_width: 1,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("center", &mut self.center),
            Parameter::new("diameter", &mut self.diameter),
            Parameter::new("start", &mut self.angle_start),
            Parameter::new("sweep", &mut self.angle_sweep),
            Parameter::new("stroke", &mut self.stroke_width),
        ]
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        let sector = Sector::with_center(
            self.center,
            self.diameter,
            (self.angle_start as f32).deg(),
            (self.angle_sweep as f32).deg(),
        );

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::CSS_SPRING_GREEN)
            .stroke_width(self.stroke_width)
            .fill_color(Rgb888::CSS_DARK_SEA_GREEN)
            .build();
        let styled_sector = sector.into_styled(style);

        draw::bounding_box(&styled_sector, display);
        styled_sector.draw(display)?;
        draw::point(self.center, Rgb888::CSS_LIGHT_SKY_BLUE, display);

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Sector debugger", &settings);

    SectorDebug::run(window);
}
