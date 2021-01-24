use embedded_graphics::{
    pixelcolor::Rgb565, prelude::*, primitives::Polyline, style::PrimitiveStyle,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

struct PolylineDebug {
    points: u32,
    p1: Point,
    p2: Point,
    p3: Point,
    p4: Point,
    p5: Point,
    stroke_width: u32,
}

impl App for PolylineDebug {
    type Color = Rgb565;
    const DISPLAY_SIZE: Size = Size::new(256, 256);

    fn new() -> Self {
        Self {
            points: 5,
            p1: Point::new(65, 130),
            p2: Point::new(120, 80),
            p3: Point::new(190, 120),
            p4: Point::new(190, 70),
            p5: Point::new(220, 50),
            stroke_width: 10,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("points", &mut self.points),
            Parameter::new("p1", &mut self.p1),
            Parameter::new("p2", &mut self.p2),
            Parameter::new("p3", &mut self.p3),
            Parameter::new("p4", &mut self.p4),
            Parameter::new("p5", &mut self.p5),
            Parameter::new("stroke", &mut self.stroke_width),
        ]
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        let points = [self.p1, self.p2, self.p3, self.p4, self.p5];
        let points = &points[0..(self.points as usize).min(points.len())];

        Polyline::new(points)
            .into_styled(PrimitiveStyle::with_stroke(
                Rgb565::GREEN,
                self.stroke_width,
            ))
            .draw(display)
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Polyline debugger", &settings);

    PolylineDebug::run(window);
}
