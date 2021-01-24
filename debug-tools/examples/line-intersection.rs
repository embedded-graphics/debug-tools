use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::{Rgb565, WebColors},
    prelude::*,
    primitives::{Circle, Line},
    style::{MonoTextStyle, PrimitiveStyle},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

struct LineDebug {
    l1_start: Point,
    l1_end: Point,
    l2_start: Point,
    l2_end: Point,
}

impl App for LineDebug {
    type Color = Rgb565;
    const DISPLAY_SIZE: Size = Size::new(256, 256);

    fn new() -> Self {
        Self {
            l1_start: Point::new(150, 170),
            l1_end: Point::new(170, 200),
            l2_start: Point::new(120, 130),
            l2_end: Point::new(145, 169),
        }
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        vec![
            Parameter::new("l1_start", &mut self.l1_start),
            Parameter::new("l1_end", &mut self.l1_end),
            Parameter::new("l2_start", &mut self.l2_start),
            Parameter::new("l2_end", &mut self.l2_end),
        ]
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        let line1 = Line::new(self.l1_start, self.l1_end);
        let line2 = Line::new(self.l2_start, self.l2_end);

        line1
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_STEEL_BLUE, 1))
            .draw(display)?;

        line2
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_SKY_BLUE, 1))
            .draw(display)?;

        let text = match intersection(&line1, &line2) {
            Intersection::Colinear => "colinear".to_string(),
            Intersection::Point {
                point,
                outer_side,
                is_special_case,
            } => {
                let point_color = if is_special_case {
                    Rgb565::CSS_TOMATO
                } else {
                    Rgb565::CSS_SPRING_GREEN
                };

                Circle::with_center(point, 3)
                    .into_styled(PrimitiveStyle::with_stroke(point_color, 1))
                    .draw(display)?;

                format!("Point: ({}, {}), {:?}", point.x, point.y, outer_side)
            }
        };

        Text::new(&text, Point::new(12, 40))
            .into_styled(MonoTextStyle::new(Font6x8, Rgb565::WHITE))
            .draw(display)
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(3).build();
    let window = Window::new("Line intersection debugger", &settings);

    LineDebug::run(window);
}

// -------------------------------------------------------------------------------------------------
// Copied code from e-g, because these types aren't public
// -------------------------------------------------------------------------------------------------

/// Intersection test result.
#[derive(Copy, Clone, Debug)]
pub enum Intersection {
    /// Intersection at point
    Point {
        /// Intersection point.
        point: Point,

        /// The "outer" side of the intersection, i.e. the side that has the joint's reflex angle.
        ///
        /// For example:
        ///
        /// ```text
        /// # Left outer side:
        ///
        ///  ⎯
        /// ╱
        ///
        /// # Right outer side:
        ///  │
        /// ╱
        /// ```
        ///
        /// This is used to find the outside edge of a corner.
        outer_side: LineSide,

        is_special_case: bool,
    },

    /// No intersection: lines are colinear or parallel.
    Colinear,
}

/// Integer-only line intersection ///
/// Inspired from https://stackoverflow.com/a/61485959/383609, which links to
/// https://webdocs.cs.ualberta.ca/~graphics/books/GraphicsGems/gemsii/xlines.c
fn intersection(self_: &Line, other: &Line) -> Intersection {
    let line1 = LinearEquation::from_line(self_);
    let line2 = LinearEquation::from_line(other);

    // Calculate the determinant to solve the system of linear equations using Cramer's rule.
    let denominator = line1.normal_vector.determinant(line2.normal_vector);

    // The system of linear equations has no solutions if the determinant is zero. In this case,
    // the lines must be colinear.
    if denominator == 0 {
        return Intersection::Colinear;
    }

    let outer_side = if denominator > 0 {
        LineSide::Right
    } else {
        LineSide::Left
    };

    // Special case: If the two lines are almost parallel, return the average point between
    // them.
    if denominator.pow(2) < self_.delta().dot_product(other.delta()) {
        return Intersection::Point {
            point: (self_.end + other.start) / 2,
            outer_side,
            is_special_case: true,
        };
    }

    // If we got here, line segments intersect. Compute intersection point using method similar
    // to that described here: http://paulbourke.net/geometry/pointlineplane/#i2l

    // The denominator/2 is to get rounding instead of truncating.
    let offset = denominator.abs() / 2;

    let origin_distances = Point::new(line1.origin_distance, line2.origin_distance);

    let numerator =
        origin_distances.determinant(Point::new(line1.normal_vector.y, line2.normal_vector.y));
    let x_numerator = if numerator < 0 {
        numerator - offset
    } else {
        numerator + offset
    };

    let numerator =
        Point::new(line1.normal_vector.x, line2.normal_vector.x).determinant(origin_distances);
    let y_numerator = if numerator < 0 {
        numerator - offset
    } else {
        numerator + offset
    };

    Intersection::Point {
        point: Point::new(x_numerator, y_numerator) / denominator,
        outer_side,
        is_special_case: false,
    }
}

/// Linear equation.
///
/// The equation is stored as a normal vector and the distance to the origin.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct LinearEquation {
    /// Normal vector, perpendicular to the line.
    ///
    /// The unit vector is scaled up to increase the resolution.
    pub normal_vector: Point,

    /// Distance from the origin.
    ///
    /// The distance doesn't directly correlate to the distance in pixels, but is
    /// scaled up by the length of the normal vector.
    pub origin_distance: i32,
}

impl LinearEquation {
    /// Creates a new linear equation from a line.
    pub fn from_line(line: &Line) -> Self {
        let normal_vector = line.delta().rotate_90();
        let origin_distance = line.start.dot_product(normal_vector);

        Self {
            normal_vector,
            origin_distance,
        }
    }

    /// Returns the distance between the line and a point.
    ///
    /// The scaling of the returned value depends on the length of the normal vector.
    /// Positive values will be returned for points on the left side of the line and negative
    /// values for points on the right.
    pub fn distance(&self, point: Point) -> i32 {
        point.dot_product(self.normal_vector) - self.origin_distance
    }

    /// Checks if a point is on the given side of the line.
    ///
    /// Always returns `true` if the point is on the line.
    pub fn check_side(&self, point: Point, side: LineSide) -> bool {
        let distance = self.distance(point);

        match side {
            LineSide::Right => distance <= 0,
            LineSide::Left => distance >= 0,
        }
    }
}

trait PointExt {
    /// Returns a point that is rotated by 90° relative to the origin.
    fn rotate_90(self) -> Self;

    /// Calculates the dot product of two points.
    fn dot_product(self, other: Point) -> i32;

    /// Calculates the determinant of a 2x2 matrix formed by this and another point.
    ///
    /// ```text
    ///          | self.x  self.y  |
    /// result = |                 |
    ///          | other.x other.y |
    /// ```
    fn determinant(self, other: Point) -> i32;

    /// Returns the squared length.
    ///
    /// The returned value is the square of the length of a vector from `(0, 0)`
    /// to `(self.x, self.y)`.
    fn length_squared(self) -> i32;
}

impl PointExt for Point {
    fn rotate_90(self) -> Self {
        Self::new(self.y, -self.x)
    }

    fn dot_product(self, other: Point) -> i32 {
        self.x * other.x + self.y * other.y
    }

    fn determinant(self, other: Point) -> i32 {
        self.x * other.y - self.y * other.x
    }

    fn length_squared(self) -> i32 {
        self.x.pow(2) + self.y.pow(2)
    }
}

/// Which side of the center line to draw on.
///
/// Imagine standing on `start`, looking ahead to where `end` is. `Left` is to your left, `Right` to
/// your right.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum LineSide {
    /// Left side of the line
    Left,

    /// Right side of the line
    Right,
}
