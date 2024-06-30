use embedded_graphics::{
    geometry::PointExt, mock_display::MockDisplay, pixelcolor::Rgb888, prelude::*, primitives::Line,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

#[derive(Debug, Clone, Copy)]
struct MajorMinor<T> {
    major: T,
    minor: T,
}

impl<T> MajorMinor<T> {
    fn new(major: T, minor: T) -> Self {
        Self { major, minor }
    }
}

fn thickline(
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
    line: Line,
    width: i32,
) -> Result<(), std::convert::Infallible> {
    if width == 0 {
        return Ok(());
    }

    let non_mul_line = line;
    let non_mul_delta = line.delta();

    // let mut seed_line = line.perpendicular();
    let mut line = line;
    line.start *= 256;
    line.end *= 256;

    let seed_line_delta = line.delta();

    let seed_line_direction = Point::new(
        if seed_line_delta.x >= 0 { 1 } else { -1 },
        if seed_line_delta.y >= 0 { 1 } else { -1 },
    );

    let y_major = non_mul_delta.y.abs() >= non_mul_delta.x.abs();

    let (non_mul_majorminor, seed_line_delta, seed_line_step) = if y_major {
        (
            MajorMinor::new(non_mul_delta.y, non_mul_delta.x),
            MajorMinor::new(seed_line_delta.y, seed_line_delta.x),
            MajorMinor::new(seed_line_direction.y_axis(), seed_line_direction.x_axis()),
        )
    } else {
        (
            MajorMinor::new(non_mul_delta.x, non_mul_delta.y),
            MajorMinor::new(seed_line_delta.x, seed_line_delta.y),
            MajorMinor::new(seed_line_direction.x_axis(), seed_line_direction.y_axis()),
        )
    };

    // let mut point = non_mul_line.start;
    let mut point = line.start;

    let dx = seed_line_delta.major.abs();
    let dy = seed_line_delta.minor.abs();

    // Using non-multiplied line delta otherwise thickness threshold runs into overflow issues (I
    // think? It ended up negative in testing)
    let non_mul_dx = non_mul_majorminor.major.abs();
    let non_mul_dy = non_mul_majorminor.minor.abs();

    let threshold = dx - 2 * dy;
    // http://kt8216.unixcab.org/murphy/index.html calls e_minor E_diag, and e_major E_square
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let mut seed_line_error = 0;

    // Subtract 1 if using AA so 1px wide lines are _only_ drawn with AA - no solid fill
    let thickness_threshold = ((width - 1) * 2).pow(2) * non_mul_delta.length_squared();
    // Add the first line drawn to the thickness. If this is left at zero, an extra line will be
    // drawn as the lines are drawn before checking for thickness.
    let mut thickness_accumulator = 2 * non_mul_dx;

    println!(
        "thresh {} thick thresh {} e_minor {} e_major {} dx {} dy {}",
        threshold, thickness_threshold, e_minor, e_major, dx, dy
    );

    while thickness_accumulator.pow(2) <= thickness_threshold {
        println!("error {} point {}", seed_line_error, point);

        let c = Rgb888::CSS_FOREST_GREEN;

        Pixel(Point::new(point.x >> 8, point.y >> 8), c).draw(display)?;

        // We seem to hit the threshold too early and end up with a 45 degree line everywhere.
        if seed_line_error > threshold {
            point += seed_line_step.minor * 256;
            seed_line_error += e_minor;
            thickness_accumulator += 2 * non_mul_dy;
        }

        point += seed_line_step.major * 256;
        seed_line_error += e_major;
        thickness_accumulator += 2 * non_mul_dx;
    }

    Ok(())
}

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
}

impl App for LineDebug {
    type Color = Rgb888;
    const DISPLAY_SIZE: Size = Size::new(200, 200);
    // const DISPLAY_SIZE: Size = Size::new(64, 64);

    fn new() -> Self {
        let end = Point::new(
            Self::DISPLAY_SIZE.width as i32 / 2,
            Self::DISPLAY_SIZE.height as i32 / 2,
        );
        Self {
            start: end - Point::new(80, 35),
            end,
            // end: start + Point::new(100, 0),
            stroke_width: 20,
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
        let Point { x: _x0, y: _y0 } = self.start;

        // let width = 2 * self.stroke_width as i32 * f32::sqrt((dx * dx + dy * dy) as f32) as i32;
        // let width = (self.stroke_width as i32).pow(2) * (dx * dx + dy * dy);
        let width = self.stroke_width as i32;

        let _mock_display: MockDisplay<Rgb888> = MockDisplay::new();

        thickline(display, Line::new(self.start, self.end), width)?;

        // let l = Line::new(self.start, self.end);

        // l.into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, width as u32))
        //     .draw(display)?;

        // l.perpendicular()
        //     .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
        //     .draw(&mut display.translated(Point::new(40, 40)))?;

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(5).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
