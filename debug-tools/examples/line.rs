use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Line, style::PrimitiveStyle};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
}

impl App for LineDebug {
    type Color = Rgb565;
    const DISPLAY_SIZE: Size = Size::new(256, 256);

    fn new() -> Self {
        Self {
            start: Point::new(128, 128),
            end: Point::new(150, 170),
            stroke_width: 1,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("start", &mut self.start),
            Parameter::new("end", &mut self.end),
            Parameter::new("stroke", &mut self.stroke_width),
        ]
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        Line::new(self.start, self.end)
            .into_styled(PrimitiveStyle::with_stroke(
                Rgb565::GREEN,
                self.stroke_width,
            ))
            .draw(display)
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
