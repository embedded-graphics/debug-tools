use embedded_graphics::{
    pixelcolor::{Gray4, Gray8, Rgb888},
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::{draw, prelude::*};

struct CircleDebug {
    center: Point,
    diameter: u32,
    stroke_width: u32,
    show_bounding_box: bool,
}

impl App for CircleDebug {
    type Color = Rgb888;
    const DISPLAY_SIZE: Size = Size::new(128, 128);

    fn new() -> Self {
        Self {
            center: Point::new(64, 80),
            diameter: 70,
            stroke_width: 1,
            show_bounding_box: false,
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("center", &mut self.center),
            Parameter::new("diameter", &mut self.diameter),
            Parameter::new("stroke", &mut self.stroke_width),
            Parameter::new("show BB", &mut self.show_bounding_box),
        ]
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        let circle = Circle::with_center(self.center, self.diameter);

        let bb = circle.bounding_box();
        let center = bb.center();

        let max_distance = bb.size.width as f32 * 2.0f32.sqrt();

        for point in bb.points() {
            let distance_to_center =
                f32::sqrt(((point.x - center.x).pow(2) + (point.y - center.y).pow(2)) as f32);

            let distance_to_edge = distance_to_center - circle.diameter as f32 / 2.0;

            let norm = distance_to_center / max_distance;

            let scaled = (norm * 255.0) as u8;

            // Distance field
            {
                let grey = Gray8::new(scaled);

                let color = Rgb888::from(grey);

                Pixel(point, color).draw(display)?;
            }

            // // Non antialiased circle
            // {
            //     let color = if distance_to_edge < 0.0 {
            //         Rgb888::RED
            //     } else {
            //         Rgb888::GREEN
            //     };

            //     Pixel(point, color).draw(display)?;
            // }

            // Antialiased circle
            {
                let color = (0.5 - distance_to_edge).clamp(0.0, 1.0);

                let scaled = (color * 255.0) as u8;

                // Kludge for "transparent" pixels
                if scaled == 0 {
                    continue;
                }

                let color = Rgb888::new(scaled, 0, 0);

                Pixel(point, color).draw(display)?;
            }
        }

        // let style = PrimitiveStyleBuilder::new()
        //     .stroke_color(Rgb888::CSS_SPRING_GREEN)
        //     .stroke_width(self.stroke_width)
        //     .fill_color(Rgb888::CSS_DARK_SEA_GREEN)
        //     .build();
        // let styled_circle = circle.into_styled(style);

        // if self.show_bounding_box {
        //     draw::bounding_box(&styled_circle, display);
        // }

        // styled_circle.draw(display)?;

        // if self.show_bounding_box {
        //     draw::point(self.center, Rgb888::CSS_LIGHT_SKY_BLUE, display);
        // }

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();
    let window = Window::new("Circle debugger", &settings);

    CircleDebug::run(window);
}
