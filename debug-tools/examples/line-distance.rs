use embedded_graphics::{
    pixelcolor::{Gray8, Rgb888},
    prelude::*,
    primitives::{line::StrokeOffset, Line, PrimitiveStyle},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
}

impl App for LineDebug {
    type Color = Rgb888;
    const DISPLAY_SIZE: Size = Size::new(256, 256);

    fn new() -> Self {
        Self {
            start: Point::new(128, 128),
            end: Point::new(150, 170),
            stroke_width: 15,
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
        let skeleton = Line::new(self.start, self.end);
        let (l, r) = skeleton.extents(self.stroke_width, StrokeOffset::None);

        // Structure
        {
            skeleton
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
                .draw(display)?;
            l.into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
                .draw(display)?;
            r.into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 1))
                .draw(display)?;
        }

        let length = {
            let Point { x, y } = skeleton.delta();

            f32::sqrt((x.pow(2) + y.pow(2)) as f32)
        };

        let bb = skeleton
            .into_styled(PrimitiveStyle::with_stroke(
                Rgb888::BLACK,
                self.stroke_width,
            ))
            .bounding_box();

        let max_distance = bb.size.width.max(bb.size.height) as f32 * 2.0f32.sqrt();

        // for point in l.points() {
        for point in bb.points() {
            // http://paulbourke.net/geometry/pointlineplane
            let distance = {
                let x1 = skeleton.start.x;
                let x2 = skeleton.end.x;
                let x3 = point.x;

                let y1 = skeleton.start.y;
                let y2 = skeleton.end.y;
                let y3 = point.y;

                let u = ((x3 - x1) * (x2 - x1) + (y3 - y1) * (y2 - y1)) as f32 / length.powi(2);

                let tx = x1 as f32 + u * (x2 - x1) as f32;
                let ty = y1 as f32 + u * (y2 - y1) as f32;

                // Tangent intersection point
                let tangent = Point::new(tx as i32, ty as i32);

                let Point { x, y } = Line::new(point, tangent).delta();

                f32::sqrt((x.pow(2) + y.pow(2)) as f32)
            };

            let norm = distance / max_distance;

            let gray = Gray8::new(unsafe { (norm * 255.0).to_int_unchecked() });
            let color = Rgb888::from(gray);

            Pixel(point, color).draw(display)?;
        }

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
