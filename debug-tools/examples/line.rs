use embedded_graphics::{
    pixelcolor::{Gray8, Rgb565},
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

struct LineDebug {
    start: Point,
    end: Point,
    stroke_width: u32,
}

fn thin_octant1(
    display: &mut SimulatorDisplay<Rgb565>,
    x0: i32,
    y0: i32,
    dx: i32,
    dy: i32,
) -> Result<(), std::convert::Infallible> {
    let mut error: i32 = 0;
    let mut y = y0;
    let mut x = x0;
    let threshold = dx - 2 * dy;
    let e_major = 2 * dx;
    let e_minor = 2 * dy;
    let length = dx;

    for _ in 1..length {
        // Pixel(Point::new(x, y - 3), Rgb565::new(ass as u8, 0, 0)).draw(display)?;
        Pixel(Point::new(x, y), Rgb565::GREEN).draw(display)?;

        if error > threshold {
            y += 1;
            error -= e_major;
        }

        error += e_minor;
        x += 1;
    }

    Ok(())
}

fn plot(
    display: &mut SimulatorDisplay<Rgb565>,
    x: f32,
    y: f32,
    c: f32,
) -> Result<(), std::convert::Infallible> {
    let c = Rgb565::new(0, (c * 255.0) as u8, 0);

    Pixel(Point::new(x as i32, y as i32), c).draw(display)
}

// integer part of x
fn ipart(x: f32) -> f32 {
    f32::floor(x)
}

fn round(x: f32) -> f32 {
    f32::trunc(x + 0.5)
}

// fractional part of x
fn fpart(x: f32) -> f32 {
    x - f32::floor(x)
}

fn rfpart(x: f32) -> f32 {
    1.0 - f32::fract(x)
}

fn wu(
    display: &mut SimulatorDisplay<Rgb565>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> Result<(), std::convert::Infallible> {
    let mut x0 = x0 as f32;
    let mut y0 = y0 as f32;
    let mut x1 = x1 as f32;
    let mut y1 = y1 as f32;

    let mut steep = f32::abs(y1 - y0) > f32::abs(x1 - x0);

    if steep {
        core::mem::swap(&mut x0, &mut y0);
        core::mem::swap(&mut x1, &mut y1);
    }
    if x0 > x1 {
        core::mem::swap(&mut x0, &mut x1);
        core::mem::swap(&mut y0, &mut y1);
    }

    let mut dx = x1 - x0;
    let mut dy = y1 - y0;
    let mut gradient = dy / dx;
    if dx == 0.0 {
        let mut gradient = 1.0;
    }

    // handle first endpoint
    let mut xend = round(x0);
    let mut yend = y0 + gradient * (xend - x0);
    let mut xgap = rfpart(x0 + 0.5);
    let mut xpxl1 = xend; // this will be used in the main loop
    let mut ypxl1 = ipart(yend);
    if steep {
        plot(display, ypxl1, xpxl1, rfpart(yend) * xgap)?;
        plot(display, ypxl1 + 1.0, xpxl1, fpart(yend) * xgap)?;
    } else {
        plot(display, xpxl1, ypxl1, rfpart(yend) * xgap)?;
        plot(display, xpxl1, ypxl1 + 1.0, fpart(yend) * xgap)?;
    }
    let mut intery = yend + gradient; // first y-intersection for the main loop

    // handle second endpoint
    let mut xend = round(x1);
    let mut yend = y1 + gradient * (xend - x1);
    let mut xgap = fpart(x1 + 0.5);
    let mut xpxl2 = xend; //this will be used in the main loop
    let mut ypxl2 = ipart(yend);
    if steep {
        plot(display, ypxl2, xpxl2, rfpart(yend) * xgap)?;
        plot(display, ypxl2 + 1.0, xpxl2, fpart(yend) * xgap)?;
    } else {
        plot(display, xpxl2, ypxl2, rfpart(yend) * xgap)?;
        plot(display, xpxl2, ypxl2 + 1.0, fpart(yend) * xgap)?;
    }

    // main loop
    if steep {
        for x in (xpxl1 as i32 + 1)..(xpxl2 as i32 - 1) {
            plot(display, ipart(intery), x as f32, rfpart(intery))?;
            plot(display, ipart(intery) + 1.0, x as f32, fpart(intery))?;
            intery = intery + gradient;
        }
    } else {
        for x in (xpxl1 as i32 + 1)..(xpxl2 as i32 - 1) {
            plot(display, x as f32, ipart(intery), rfpart(intery))?;
            plot(display, x as f32, ipart(intery) + 1.0, fpart(intery))?;
            intery = intery + gradient;
        }
    }

    Ok(())
}

// //returns 1 - fractional part of number
// fn rfPartOfNumber(x: f32) -> f32 {
//     return 1.0 - f32::fract(x);
// }

// fn drawPixel(
//     display: &mut SimulatorDisplay<Rgb565>,
//     x: f32,
//     y: f32,
//     color: f32,
// ) -> Result<(), std::convert::Infallible> {
//     Pixel(
//         Point::new(x as i32, y as i32),
//         Rgb565::new(0, (color * 255.0) as u8, 0),
//     )
//     .draw(display)
// }

// fn drawAALine(
//     display: &mut SimulatorDisplay<Rgb565>,
//     mut x0: i32,
//     mut y0: i32,
//     mut x1: i32,
//     mut y1: i32,
// ) -> Result<(), std::convert::Infallible> {
//     let steep = (y1 - y0).abs() > (x1 - x0).abs();

//     // swap the co-ordinates if slope > 1 or we
//     // draw backwards
//     if steep {
//         core::mem::swap(&mut x0, &mut y0);
//         core::mem::swap(&mut x1, &mut y1);
//     }
//     if x0 > x1 {
//         core::mem::swap(&mut x0, &mut x1);
//         core::mem::swap(&mut y0, &mut y1);
//     }

//     //compute the slope
//     let dx = x1 - x0;
//     let dy = y1 - y0;
//     let mut gradient: f32 = dy as f32 / dx as f32;
//     if dx == 0 {
//         gradient = 1.0;
//     }

//     let xpxl1 = x0;
//     let xpxl2 = x1;
//     let mut intersectY = y0 as f32;

//     // main loop
//     if steep {
//         for x in xpxl1..=xpxl2 {
//             // pixel coverage is determined by fractional
//             // part of y co-ordinate
//             drawPixel(
//                 display,
//                 f32::trunc(intersectY),
//                 x as f32,
//                 rfPartOfNumber(intersectY),
//             )?;
//             drawPixel(
//                 display,
//                 f32::trunc(intersectY) - 1.0,
//                 x as f32,
//                 f32::fract(intersectY),
//             )?;
//             intersectY += gradient;
//         }
//     } else {
//         for x in xpxl1..=xpxl2 {
//             // pixel coverage is determined by fractional
//             // part of y co-ordinate
//             drawPixel(
//                 display,
//                 x as f32,
//                 f32::trunc(intersectY),
//                 rfPartOfNumber(intersectY),
//             )?;
//             drawPixel(
//                 display,
//                 x as f32,
//                 f32::trunc(intersectY) - 1.0,
//                 f32::fract(intersectY),
//             )?;
//             intersectY += gradient;
//         }
//     }

//     Ok(())
// }

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
        let line = Line::new(self.start, self.end);

        let delta = line.delta();

        // thin_octant1(display, line.start.x, line.start.y, delta.x, delta.y)?;
        wu(display, line.start.x, line.start.y, delta.x, delta.y)?;

        // {
        //     let Line {
        //         start: Point { x: x0, y: y0 },
        //         end: Point { x: x1, y: y1 },
        //     } = line;

        //     drawAALine(display, x0, y0, x1, y1)?;
        // }

        // Line::new(self.start, self.end)
        //     .points()
        //     .map(|point| Pixel(point, Rgb565::GREEN))
        //     .draw(display)

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Line debugger", &settings);

    LineDebug::run(window);
}
