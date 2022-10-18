use embedded_graphics::{
    pixelcolor::{Gray8, Rgb888},
    prelude::*,
    primitives::{
        line::{Scanline, StrokeOffset},
        Line, PrimitiveStyle,
    },
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
}

fn distance(line: Line, point: Point) -> f32 {
    let length = {
        let Point { x, y } = line.delta();

        f32::sqrt((x.pow(2) + y.pow(2)) as f32)
    };

    let x1 = line.start.x;
    let x2 = line.end.x;
    let x3 = point.x;

    let y1 = line.start.y;
    let y2 = line.end.y;
    let y3 = point.y;

    let u = ((x3 - x1) * (x2 - x1) + (y3 - y1) * (y2 - y1)) as f32 / length.powi(2);

    let tx = x1 as f32 + u * (x2 - x1) as f32;
    let ty = y1 as f32 + u * (y2 - y1) as f32;

    // Tangent intersection point
    let tangent = Point::new(tx as i32, ty as i32);

    let Point { x, y } = Line::new(point, tangent).delta();

    f32::sqrt((x.pow(2) + y.pow(2)) as f32)
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

        let bb = skeleton
            .into_styled(PrimitiveStyle::with_stroke(
                Rgb888::BLACK,
                self.stroke_width,
            ))
            .bounding_box();
        let br = bb.bottom_right().unwrap();

        let max_distance = bb.size.width.max(bb.size.height) as f32 * 2.0f32.sqrt();

        // 4 lines that construct the perimiter of the thick line
        let perimiter = [l, Line::new(l.end, r.end), r, Line::new(r.start, l.start)];

        // Draw perimiter
        for line in &perimiter {
            line.into_styled(PrimitiveStyle::with_stroke(Rgb888::MAGENTA, 1))
                .draw(display)?;
        }

        for y in bb.top_left.y..=br.y {
            // ---

            // // Scanline (integer) intersection
            // {
            // let mut min_x = i32::MAX;
            // let mut max_x = i32::MIN;

            // for line in &perimiter {
            //     let mut scanline = Scanline::new_empty(y);
            //     scanline.bresenham_intersection(&line);

            //     if scanline.is_empty() {
            //         continue;
            //     }

            //     min_x = min_x.min(scanline.x.start);
            //     max_x = max_x.max(scanline.x.end);
            // }

            // let scanline = Scanline::new(y, min_x..max_x);
            // scanline.draw(display, Rgb888::YELLOW)?;
            // }

            //  ---

            // // Distance field
            // {
            //     let scanline = Line::new(Point::new(bb.top_left.x, y), Point::new(br.x, y));

            //     for point in scanline.points() {
            //         let distance = distance(skeleton, point);

            //         let norm = distance / max_distance;

            //         let color = Gray8::new(unsafe { (norm * 255.0).to_int_unchecked() });

            //         let color = Rgb888::from(color);

            //         Pixel(point, color).draw(display)?;
            //     }
            // }
        }

        // Crappy distance function
        // for point in bb.points() {
        //     // http://paulbourke.net/geometry/pointlineplane
        //     // Distance between skeleton and current point
        //     let distance = {
        //         let x1 = skeleton.start.x;
        //         let x2 = skeleton.end.x;
        //         let x3 = point.x;

        //         let y1 = skeleton.start.y;
        //         let y2 = skeleton.end.y;
        //         let y3 = point.y;

        //         let u = ((x3 - x1) * (x2 - x1) + (y3 - y1) * (y2 - y1)) as f32 / length.powi(2);

        //         let tx = x1 as f32 + u * (x2 - x1) as f32;
        //         let ty = y1 as f32 + u * (y2 - y1) as f32;

        //         // Tangent intersection point
        //         let tangent = Point::new(tx as i32, ty as i32);

        //         let Point { x, y } = Line::new(point, tangent).delta();

        //         f32::sqrt((x.pow(2) + y.pow(2)) as f32)
        //     };

        //     let distance_to_edge = distance - self.stroke_width as f32;

        //     let color = (0.5 - distance_to_edge).clamp(0.0, 1.0);

        //     let color = Rgb888::from(Gray8::new(unsafe { (color * 255.0).to_int_unchecked() }));

        //     Pixel(point, color).draw(display)?;
        // }

        // // Structure
        // {
        //     skeleton
        //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
        //         .draw(display)?;
        //     l.into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
        //         .draw(display)?;
        //     r.into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 1))
        //         .draw(display)?;
        // }

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
