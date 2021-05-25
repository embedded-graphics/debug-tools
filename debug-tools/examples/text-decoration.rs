use embedded_graphics::{
    mono_font::{ascii::*, MonoFont, MonoTextStyleBuilder},
    pixelcolor::Rgb888,
    prelude::*,
    text::Text,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use framework::prelude::*;

struct Face {
    name: &'static str,
    default: MonoFont<'static>,
    italic: Option<MonoFont<'static>>,
    bold: Option<MonoFont<'static>>,
}

struct TextDecoration {}

impl App for TextDecoration {
    type Color = Rgb888;
    const DISPLAY_SIZE: Size = Size::new(600, 450);

    fn new() -> Self {
        Self {}
    }

    fn parameters(&mut self) -> Vec<Parameter> {
        Vec::new()
    }

    fn draw(
        &self,
        display: &mut SimulatorDisplay<Self::Color>,
    ) -> Result<(), std::convert::Infallible> {
        let fonts = vec![
            Face {
                name: "FONT_4X6",
                default: FONT_4X6,
                italic: None,
                bold: None,
            },
            Face {
                name: "FONT_5X7",
                default: FONT_5X7,
                italic: None,
                bold: None,
            },
            Face {
                name: "FONT_5X8",
                default: FONT_5X8,
                italic: None,
                bold: None,
            },
            Face {
                name: "FONT_6X9",
                default: FONT_6X9,
                italic: None,
                bold: None,
            },
            Face {
                name: "FONT_6X10",
                default: FONT_6X10,
                italic: None,
                bold: None,
            },
            Face {
                name: "FONT_6X12",
                default: FONT_6X12,
                italic: None,
                bold: None,
            },
            Face {
                name: "FONT_6X13",
                default: FONT_6X13,
                italic: Some(FONT_6X13_ITALIC),
                bold: Some(FONT_6X13_BOLD),
            },
            Face {
                name: "FONT_7X13",
                default: FONT_7X13,
                italic: Some(FONT_7X13_ITALIC),
                bold: Some(FONT_7X13_BOLD),
            },
            Face {
                name: "FONT_7X14",
                default: FONT_7X14,
                italic: Some(FONT_7X13_ITALIC),
                bold: Some(FONT_7X14_BOLD),
            },
            Face {
                name: "FONT_8X13",
                default: FONT_8X13,
                italic: Some(FONT_8X13_ITALIC),
                bold: Some(FONT_8X13_BOLD),
            },
            Face {
                name: "FONT_9X15",
                default: FONT_9X15,
                italic: None,
                bold: Some(FONT_9X15_BOLD),
            },
            Face {
                name: "FONT_9X18",
                default: FONT_9X18,
                italic: None,
                bold: Some(FONT_9X18_BOLD),
            },
            Face {
                name: "FONT_10X20",
                default: FONT_10X20,
                italic: None,
                bold: None,
            },
        ];

        let text = "ABCabc[]\"qypilo";

        let mut position = Point::new(0, 10);

        for Face {
            name,
            default,
            bold,
            italic,
        } in fonts.iter()
        {
            let default_style = MonoTextStyleBuilder::new()
                .font(default)
                .text_color(Rgb888::WHITE)
                .build();

            let strikethrough = MonoTextStyleBuilder::from(&default_style)
                .strikethrough()
                .build();
            let underline = MonoTextStyleBuilder::from(&default_style)
                .underline()
                .build();

            position = Text::new(&format!("{} ", name), position, default_style).draw(display)?;
            position = Text::new(text, position, default_style).draw(display)?;
            position = Text::new(" ", position, default_style).draw(display)?;
            position = Text::new(text, position, strikethrough).draw(display)?;
            position = Text::new(" ", position, default_style).draw(display)?;
            position = Text::new(text, position, underline).draw(display)?;

            // Reset to next line
            position.x = 0;
            position.y += default.character_size.height as i32 + 5;

            if let Some(bold) = bold {
                let normal = MonoTextStyleBuilder::from(&default_style)
                    .font(bold)
                    .build();
                let strikethrough = MonoTextStyleBuilder::from(&normal).strikethrough().build();
                let underline = MonoTextStyleBuilder::from(&normal).underline().build();

                position = Text::new(&format!("{} bold ", name), position, normal).draw(display)?;
                position = Text::new(text, position, normal).draw(display)?;
                position = Text::new(" ", position, normal).draw(display)?;
                position = Text::new(text, position, strikethrough).draw(display)?;
                position = Text::new(" ", position, normal).draw(display)?;
                position = Text::new(text, position, underline).draw(display)?;

                // Reset to next line
                position.x = 0;
                position.y += default.character_size.height as i32 + 5;
            }

            if let Some(italic) = italic {
                let normal = MonoTextStyleBuilder::from(&default_style)
                    .font(italic)
                    .build();
                let strikethrough = MonoTextStyleBuilder::from(&normal).strikethrough().build();
                let underline = MonoTextStyleBuilder::from(&normal).underline().build();

                position =
                    Text::new(&format!("{} italic ", name), position, normal).draw(display)?;
                position = Text::new(text, position, normal).draw(display)?;
                position = Text::new(" ", position, normal).draw(display)?;
                position = Text::new(text, position, strikethrough).draw(display)?;
                position = Text::new(" ", position, normal).draw(display)?;
                position = Text::new(text, position, underline).draw(display)?;

                // Reset to next line
                position.x = 0;
                position.y += default.character_size.height as i32 + 5;
            }
        }

        Ok(())
    }
}

fn main() {
    let settings = OutputSettingsBuilder::new().scale(2).build();
    let window = Window::new("Builtin font decoration debugger", &settings);

    TextDecoration::run(window);
}
