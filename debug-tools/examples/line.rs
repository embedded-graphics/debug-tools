use embedded_graphics::{
    mock_display::MockDisplay,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

fn thin_octant1(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    x0: i32,
    y0: i32,
    dx: i32,
    dy: i32,
) -> Result<(), std::convert::Infallible> {
    let mut error = 0;
    let mut y = y0;
    let mut x = x0;
    let mut threshold = dx - 2 * dy;
    let mut E_diag = -2 * dx;
    let mut E_square = 2 * dy;
    let mut length = dx;

    for p in 1..length {
        Pixel(Point::new(x, y), Rgb565::GREEN).draw(display)?;

        if error > threshold {
            y += 1;
            error += E_diag;
        }
        error += E_square;
        x += 1;
    }

    Ok(())
}

fn thick_octant1(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    x0: i32,
    y0: i32,
    dx: i32,
    dy: i32,
) -> Result<(), std::convert::Infallible> {
    // the perpendicular error or 'phase'
    let mut p_error = 0;
    let mut error = 0;
    let mut y = y0;
    let mut x = x0;
    let mut threshold = dx - 2 * dy;
    let mut E_diag = -2 * dx;
    let mut E_square = 2 * dy;
    let mut length = dx;

    for p in 1..length {
        pleft_octant1(display, x, y, dx, dy, p_error)?;
        pright_octant1(display, x, y, dx, dy, p_error)?;

        if error > threshold {
            y = y + 1;
            error = error + E_diag;
            if p_error > threshold {
                pleft_octant1(display, x, y, dx, dy, p_error + E_diag + E_square)?;
                // FIXME: Overdraw
                // pright_octant1(display, x, y, dx, dy, p_error + E_diag + E_square)?;
                p_error = p_error + E_diag;
            }
            p_error = p_error + E_square;
        }
        error = error + E_square;
        x = x + 1;
    }

    Ok(())
}

fn pleft_octant1(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    x0: i32,
    y0: i32,
    dx: i32,
    dy: i32,
    mut error: i32,
) -> Result<(), std::convert::Infallible> {
    let mut y = y0;
    let mut x = x0;
    let mut threshold = dx - 2 * dy;
    let mut E_diag = -2 * dx;
    let mut E_square = 2 * dy;
    let mut thickness = 10;

    for p in 1..thickness {
        Pixel(Point::new(x, y), Rgb565::GREEN).draw(display)?;

        if error > threshold {
            x = x - 1;
            error = error + E_diag;
        }
        error = error + E_square;
        y = y + 1;
    }

    Ok(())
}

fn pright_octant1(
    display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
    x0: i32,
    y0: i32,
    dx: i32,
    dy: i32,
    mut error: i32,
) -> Result<(), std::convert::Infallible> {
    let mut y = y0;
    let mut x = x0;
    let mut threshold = dx - 2 * dy;
    let mut E_diag = -2 * dx;
    let mut E_square = 2 * dy;
    let mut thickness = 10;

    error = -error;

    for p in 1..thickness {
        if error > threshold {
            x = x + 1;
            error = error + E_diag;
        }
        error = error + E_square;
        y = y - 1;

        Pixel(Point::new(x, y), Rgb565::RED).draw(display)?;
    }

    Ok(())
}

// fn pright_octant1(
//     display: &mut impl DrawTarget<Color = Rgb565, Error = std::convert::Infallible>,
//     x0: i32,
//     y0: i32,
//     dx: i32,
//     dy: i32,
//     mut error: i32,
// ) -> Result<(), std::convert::Infallible> {
//     let mut y = y0;
//     let mut x = x0;
//     let mut threshold = dx - 2 * dy;
//     let mut E_diag = -2 * dx;
//     let mut E_square = 2 * dy;
//     let mut thickness = 10;

//     for p in 1..thickness {
//         if error < -threshold {
//             x = x + 1;
//             error = error - E_diag;
//         }
//         error = error - E_square;
//         y = y - 1;

//         Pixel(Point::new(x, y), Rgb565::RED).draw(display)?;
//     }

//     Ok(())
// }

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
}

impl App for LineDebug {
    type Color = Rgb565;
    const DISPLAY_SIZE: Size = Size::new(64, 64);

    fn new() -> Self {
        Self {
            start: Point::new(32, 32),
            end: Point::new(40, 50),
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
        let Point { x: x0, y: y0 } = self.start;
        let Point { x: x1, y: y1 } = self.end;

        let dx = x1 - x0;
        let dy = y1 - y0;

        // thin_octant1(display, x0, y0, dx, dy)?;
        thick_octant1(display, x0, y0, dx, dy)?;

        let mut mock_display = MockDisplay::new();

        thick_octant1(&mut mock_display, x0, y0, dx, dy).unwrap();

        Ok(())

        // Line::new(self.start, self.end)
        //     .into_styled(PrimitiveStyle::with_stroke(
        //         Rgb565::GREEN,
        //         self.stroke_width,
        //     ))
        //     .draw(display)
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
