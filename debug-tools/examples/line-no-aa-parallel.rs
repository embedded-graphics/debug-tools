use embedded_graphics::{
    geometry::PointExt,
    mock_display::MockDisplay,
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{
        common::{LineSide, LinearEquation},
        line::StrokeOffset,
        Line, PrimitiveStyle,
    },
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;
use integer_sqrt::IntegerSquareRoot;

#[derive(Debug, Copy, Clone, PartialEq)]
enum LineOffset {
    Left,
    Center,
    Right,
}

impl LineOffset {
    fn widths(self, width: i32) -> (i32, i32) {
        match width {
            width => {
                match self {
                    Self::Left => (width.saturating_sub(1), 0),
                    Self::Center => {
                        let width = width.saturating_sub(1);

                        // Right-side bias for even width lines. Move mod2 to first item in the
                        // tuple to bias to the left instead.
                        (width / 2, width / 2 + (width % 2))
                    }
                    Self::Right => (width.saturating_sub(1), 0),
                }
            }
        }
    }
}

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
    toggle: bool,
    toggle2: bool,
) -> Result<(), std::convert::Infallible> {
    let Line { start, end } = line;

    // let extents = line.extents(width as u32, StrokeOffset::None);
    // // The perpendicular starting edge of the line
    // let seed_line = Line::new(extents.0.start, extents.1.start);

    let seed_line = line.perpendicular();

    let parallel_delta = line.end - line.start;
    let parallel_step = Point::new(
        if parallel_delta.x >= 0 { 1 } else { -1 },
        if parallel_delta.y >= 0 { 1 } else { -1 },
    );

    // let (delta, step, pdelta, pstep) = {
    //     let delta = end - start;

    //     let direction = Point::new(
    //         if delta.x >= 0 { 1 } else { -1 },
    //         if delta.y >= 0 { 1 } else { -1 },
    //     );

    //     let perp_delta = line.perpendicular();
    //     let perp_delta = perp_delta.end - perp_delta.start;

    //     let perp_direction = Point::new(
    //         if perp_delta.x >= 0 { 1 } else { -1 },
    //         if perp_delta.y >= 0 { 1 } else { -1 },
    //     );

    //     let (perp_delta, perp_direction) = if perp_delta.y.abs() >= perp_delta.x.abs() {
    //         (
    //             MajorMinor::new(perp_delta.y, perp_delta.x),
    //             MajorMinor::new(perp_direction.y_axis(), perp_direction.x_axis()),
    //         )
    //     } else {
    //         (
    //             MajorMinor::new(perp_delta.x, perp_delta.y),
    //             MajorMinor::new(perp_direction.x_axis(), perp_direction.y_axis()),
    //         )
    //     };

    //     // Determine major and minor directions.
    //     if delta.y.abs() >= delta.x.abs() {
    //         (
    //             MajorMinor::new(delta.y, delta.x),
    //             MajorMinor::new(direction.y_axis(), direction.x_axis()),
    //             perp_delta,
    //             perp_direction,
    //         )
    //     } else {
    //         (
    //             MajorMinor::new(delta.x, delta.y),
    //             MajorMinor::new(direction.x_axis(), direction.y_axis()),
    //             perp_delta,
    //             perp_direction,
    //         )
    //     }
    // };

    let mut point = seed_line.start;

    // Base line skeleton
    // parallel_line(point, line, step, delta, 0, Rgb888::MAGENTA, display)?;

    let seed_line_delta = seed_line.end - seed_line.start;

    let seed_line_direction = Point::new(
        if seed_line_delta.x >= 0 { 1 } else { -1 },
        if seed_line_delta.y >= 0 { 1 } else { -1 },
    );

    let (seed_line_delta, seed_line_step) = if seed_line_delta.y.abs() >= seed_line_delta.x.abs() {
        (
            MajorMinor::new(seed_line_delta.y, seed_line_delta.x),
            MajorMinor::new(seed_line_direction.y_axis(), seed_line_direction.x_axis()),
        )
    } else {
        (
            MajorMinor::new(seed_line_delta.x, seed_line_delta.y),
            MajorMinor::new(seed_line_direction.x_axis(), seed_line_direction.y_axis()),
        )
    };

    let (parallel_delta, parallel_step) = if parallel_delta.y.abs() >= parallel_delta.x.abs() {
        (
            MajorMinor::new(parallel_delta.y, parallel_delta.x),
            MajorMinor::new(parallel_step.y_axis(), parallel_step.x_axis()),
        )
    } else {
        (
            MajorMinor::new(parallel_delta.x, parallel_delta.y),
            MajorMinor::new(parallel_step.x_axis(), parallel_step.y_axis()),
        )
    };

    let dx = seed_line_delta.major.abs();
    let dy = seed_line_delta.minor.abs();

    let pdx = parallel_delta.major.abs();
    let pdy = parallel_delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx + 1;
    let mut seed_line_error = 0;
    // Perpendicular error or "phase"
    let mut parallel_error = 0;

    let p_threshold = pdx - 2 * pdy;
    let p_e_minor = -2 * pdx;
    let p_e_major = 2 * pdy;

    let flip = if seed_line_step.minor == -parallel_step.major {
        -1
    } else {
        1
    };

    // TODO: Proper thickness calculation
    for i in 0..width {
        // Pixel(point, Rgb888::WHITE).draw(display)?;

        parallel_line(
            point,
            line,
            parallel_step,
            parallel_delta,
            parallel_error * flip,
            Rgb888::MAGENTA,
            display,
        )?;

        if seed_line_error > threshold {
            point += seed_line_step.minor;
            seed_line_error += e_minor;

            if parallel_error > p_threshold {
                let p = if flip == 1 {
                    point
                } else {
                    // Put point on other side of the line
                    // FIXME: This is such a hack...
                    point - seed_line_step.minor + seed_line_step.major
                };

                if toggle {
                    Pixel(p, Rgb888::CYAN).draw(display)?;

                    parallel_line(
                        point,
                        line,
                        parallel_step,
                        parallel_delta,
                        (parallel_error + p_e_minor + p_e_major) * flip,
                        Rgb888::CYAN,
                        display,
                    )?;
                }

                parallel_error += p_e_minor;
            }

            parallel_error += p_e_major;
        }

        point += seed_line_step.major/* * 3*/;
        seed_line_error += e_major;
    }

    // Pixel(line.start, Rgb888::RED).draw(display)?;

    // seed_line
    //     .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
    //     .draw(display)?;

    Ok(())
}

fn parallel_line(
    start: Point,
    line: Line,
    step: MajorMinor<Point>,
    delta: MajorMinor<i32>,
    start_error: i32,
    c: Rgb888,
    display: &mut impl DrawTarget<Color = Rgb888, Error = std::convert::Infallible>,
) -> Result<(), std::convert::Infallible> {
    let mut point = start;

    let dx = delta.major.abs();
    let dy = delta.minor.abs();

    let threshold = dx - 2 * dy;
    let e_minor = -2 * dx;
    let e_major = 2 * dy;
    let length = dx + 1;
    let mut error = start_error;

    for _i in 0..length {
        Pixel(point, c).draw(display)?;

        if error > threshold {
            point += step.minor;
            error += e_minor;
        }

        error += e_major;
        point += step.major;
    }

    Ok(())
}

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
    toggle: bool,
    toggle2: bool,
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
            start: end + Point::new(10, 15),
            end,
            // end: start + Point::new(100, 0),
            stroke_width: 10,
            toggle: true,
            toggle2: false,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("start", &mut self.start),
            Parameter::new("end", &mut self.end),
            Parameter::new("stroke", &mut self.stroke_width),
            Parameter::new("toggle", &mut self.toggle),
            Parameter::new("toggle 2", &mut self.toggle2),
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

        thickline(
            display,
            Line::new(self.start, self.end),
            width,
            self.toggle,
            self.toggle2,
        )?;

        // let l = Line::new(self.start, self.end);

        // l.into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
        //     .draw(&mut display.translated(Point::new(40, 40)))?;

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
