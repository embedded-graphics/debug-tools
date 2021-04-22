use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::{draw, prelude::*};

struct CircleDebug {
    center: Point,
    diameter: u32,
    stroke_width: u32,
}

impl App for CircleDebug {
    type Color = Rgb888;
    const DISPLAY_SIZE: Size = Size::new(256, 256);

    fn new() -> Self {
        Self {
            center: Point::new(128, 128),
            diameter: 50,
            stroke_width: 1,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("center", &mut self.center),
            Parameter::new("diameter", &mut self.diameter),
            Parameter::new("stroke", &mut self.stroke_width),
        ]
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        let circle = Circle::with_center(self.center, self.diameter);

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::CSS_SPRING_GREEN)
            .stroke_width(self.stroke_width)
            .fill_color(Rgb888::CSS_DARK_SEA_GREEN)
            .build();
        let styled_circle = circle.into_styled(style);

        draw::bounding_box(&styled_circle, display);
        styled_circle.draw(display)?;
        draw::point(self.center, Rgb888::CSS_LIGHT_SKY_BLUE, display);

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Circle debugger", &settings);

    CircleDebug::run(window);
}
