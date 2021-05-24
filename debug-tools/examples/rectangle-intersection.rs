use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

struct RectangleIntersection {
    top_left: Point,
    bottom_right: Point,
}

impl App for RectangleIntersection {
    type Color = Rgb888;
    const DISPLAY_SIZE: Size = Size::new(200, 200);

    fn new() -> Self {
        Self {
            top_left: Point::new(80, 80),
            bottom_right: Point::new(150, 150),
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("top-left", &mut self.top_left),
            Parameter::new("bottom-right", &mut self.bottom_right),
        ]
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        let base_rectangle = Rectangle::with_corners(Point::new(20, 20), Point::new(100, 100));
        let moving_rectangle = Rectangle::with_corners(self.top_left, self.bottom_right);

        base_rectangle
            .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
            .draw(display)?;

        moving_rectangle
            .into_styled(PrimitiveStyle::with_fill(Rgb888::GREEN))
            .draw(display)?;

        let intersection = base_rectangle.intersection(&moving_rectangle);

        intersection
            .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
            .draw(display)?;

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Rectangle intersection", &settings);

    RectangleIntersection::run(window);
}
