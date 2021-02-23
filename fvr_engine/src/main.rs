use anyhow::Result;

#[allow(dead_code)]
mod parser {
    use anyhow::Result;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_till1},
        multi::many1,
        sequence::tuple,
        IResult,
    };

    // Special characters.
    const NEWLINE: char = '\n';
    const LEFT_CHEVRON: char = '<';

    // Special text tags.
    const NEWLINE_TAG: &str = "\n";
    const DOUBLE_LEFT_CHEVRON_TAG: &str = "<<";

    // Tags for identifying the inline format hints.
    const LEFT_CHEVRON_TAG: &str = "<";
    const COLON_TAG: &str = ":";
    const RIGHT_CHEVRON_TAG: &str = ">";

    // Tags for the possible hint keys.
    const LAYOUT_KEY_TAG: &str = "l";
    const OUTLINED_KEY_TAG: &str = "o";
    const FOREGROUND_COLOR_KEY_TAG: &str = "fc";
    const BACKGROUND_COLOR_KEY_TAG: &str = "bc";
    const OUTLINE_COLOR_KEY_TAG: &str = "oc";

    // Tags for the possible layout values.
    const CENTER_LAYOUT_VALUE_TAG: &str = "c";
    const FLOOR_LAYOUT_VALUE_TAG: &str = "f";
    const TEXT_LAYOUT_VALUE_TAG: &str = "t";

    // Tags for the possible boolean values.
    const TRUE_VALUE_TAG: &str = "t";
    const FALSE_VALUE_TAG: &str = "f";

    // Tags for the possible color values.
    const DARK_RED_COLOR_VALUE_TAG: &str = "r";
    const BRIGHT_RED_COLOR_VALUE_TAG: &str = "R";
    const DARK_ORANGE_COLOR_VALUE_TAG: &str = "o";
    const BRIGHT_ORANGE_COLOR_VALUE_TAG: &str = "O";
    const BROWN_COLOR_VALUE_TAG: &str = "w";
    const YELLOW_COLOR_VALUE_TAG: &str = "W";
    const DARK_GREEN_COLOR_VALUE_TAG: &str = "g";
    const BRIGHT_GREEN_COLOR_VALUE_TAG: &str = "G";
    const DARK_BLUE_COLOR_VALUE_TAG: &str = "b";
    const BRIGHT_BLUE_COLOR_VALUE_TAG: &str = "B";
    const DARK_PURPLE_COLOR_VALUE_TAG: &str = "p";
    const BRIGHT_PURPLE_COLOR_VALUE_TAG: &str = "P";
    const DARK_CYAN_COLOR_VALUE_TAG: &str = "c";
    const BRIGHT_CYAN_COLOR_VALUE_TAG: &str = "C";
    const DARK_MAGENTA_COLOR_VALUE_TAG: &str = "m";
    const BRIGHT_MAGENTA_COLOR_VALUE_TAG: &str = "M";
    const GOLD_COLOR_VALUE_TAG: &str = "$";
    const BLACK_COLOR_VALUE_TAG: &str = "k";
    const DARK_GREY_COLOR_VALUE_TAG: &str = "K";
    const BRIGHT_GREY_COLOR_VALUE_TAG: &str = "y";
    const WHITE_COLOR_VALUE_TAG: &str = "Y";
    const TRANSPARENT_COLOR_VALUE_TAG: &str = "T";

    // Enum of possible types of format hints.
    #[derive(Debug, PartialEq)]
    pub enum FormatHintType {
        Layout,
        Outlined,
        ForegroundColor,
        BackgroundColor,
        OutlineColor,
    }

    // Enum of possible parsed values, which can either be text, a newline, or a format hint.
    #[derive(Debug, PartialEq)]
    pub enum ParsedValue {
        Text(String),
        Newline,
        FormatHint { key: FormatHintType, value: String },
    }

    // Parser for a string of text that does not contain a newline, escaped left chevron, or format hint.
    fn text_parser(input: &str) -> IResult<&str, ParsedValue> {
        let (remainder, result) = take_till1(|c: char| c == NEWLINE || c == LEFT_CHEVRON)(input)?;

        Ok((remainder, ParsedValue::Text(result.into())))
    }

    #[test]
    fn test_text_parser() {
        assert_eq!(
            text_parser("abcdefg"),
            Ok(("", ParsedValue::Text("abcdefg".into())))
        );
        assert_eq!(
            text_parser("abc<defg"),
            Ok(("<defg", ParsedValue::Text("abc".into())))
        );
        assert_eq!(
            text_parser("abc\ndefg"),
            Ok(("\ndefg", ParsedValue::Text("abc".into())))
        );

        let error = nom::Err::Error(nom::error::Error {
            input: "<abcdefg",
            code: nom::error::ErrorKind::TakeTill1,
        });
        assert_eq!(text_parser("<abcdefg"), Err(error));
    }

    // Parser for a single newline character.
    fn newline_parser(input: &str) -> IResult<&str, ParsedValue> {
        let (remainder, _) = tag(NEWLINE_TAG)(input)?;

        Ok((remainder, ParsedValue::Newline))
    }

    #[test]
    fn test_newline_parser() {
        assert_eq!(newline_parser("\nabc"), Ok(("abc", ParsedValue::Newline)));

        let error = nom::Err::Error(nom::error::Error {
            input: "abc\n",
            code: nom::error::ErrorKind::Tag,
        });
        assert_eq!(newline_parser("abc\n"), Err(error));
    }

    // Parser for double (escaped) left chevron, which translates to a single left chevron.
    fn escaped_chevron_parser(input: &str) -> IResult<&str, ParsedValue> {
        let (remainder, _) = tag(DOUBLE_LEFT_CHEVRON_TAG)(input)?;

        Ok((remainder, ParsedValue::Text(LEFT_CHEVRON_TAG.into())))
    }

    #[test]
    fn test_escaped_chevron_parser() {
        assert_eq!(
            escaped_chevron_parser("<<abcd"),
            Ok(("abcd", ParsedValue::Text("<".into())))
        );

        let error = nom::Err::Error(nom::error::Error {
            input: "<abcd",
            code: nom::error::ErrorKind::Tag,
        });
        assert_eq!(escaped_chevron_parser("<abcd"), Err(error));
    }

    // Parser for a single left chevron, which designates the start of a format hint.
    fn format_hint_begin_parser(input: &str) -> IResult<&str, &str> {
        tag(LEFT_CHEVRON_TAG)(input)
    }

    // Parser for the format hint key/value separator colon.
    fn format_hint_separator_parser(input: &str) -> IResult<&str, &str> {
        tag(COLON_TAG)(input)
    }

    // Parser for a single right chevron, which designates the end of a format hint.
    fn format_hint_end_parser(input: &str) -> IResult<&str, &str> {
        tag(RIGHT_CHEVRON_TAG)(input)
    }

    // Parses the value of a layout format hint.
    fn layout_value_parser(input: &str) -> IResult<&str, &str> {
        alt((
            tag(CENTER_LAYOUT_VALUE_TAG),
            tag(FLOOR_LAYOUT_VALUE_TAG),
            tag(TEXT_LAYOUT_VALUE_TAG),
        ))(input)
    }

    #[test]
    fn test_layout_value_parser() {
        assert_eq!(layout_value_parser("c"), Ok(("", "c")));
        assert_eq!(layout_value_parser("f"), Ok(("", "f")));
        assert_eq!(layout_value_parser("t"), Ok(("", "t")));

        let error = nom::Err::Error(nom::error::Error {
            input: "z",
            code: nom::error::ErrorKind::Tag,
        });
        assert_eq!(layout_value_parser("z"), Err(error));
    }

    // Parser that matches a layout format hint.
    fn layout_hint_parser(input: &str) -> IResult<&str, ParsedValue> {
        let (remainder, result) = tuple((
            format_hint_begin_parser,
            tag(LAYOUT_KEY_TAG),
            format_hint_separator_parser,
            layout_value_parser,
            format_hint_end_parser,
        ))(input)?;

        Ok((
            remainder,
            ParsedValue::FormatHint {
                key: FormatHintType::Layout,
                value: result.3.into(),
            },
        ))
    }

    #[test]
    fn test_layout_hint_parser() {
        let format_hint = ParsedValue::FormatHint {
            key: FormatHintType::Layout,
            value: "c".into(),
        };
        assert_eq!(layout_hint_parser("<l:c>"), Ok(("", format_hint)));

        let format_hint = ParsedValue::FormatHint {
            key: FormatHintType::Layout,
            value: "f".into(),
        };
        assert_eq!(layout_hint_parser("<l:f>Hello"), Ok(("Hello", format_hint)));

        let format_hint = ParsedValue::FormatHint {
            key: FormatHintType::Layout,
            value: "t".into(),
        };
        assert_eq!(layout_hint_parser("<l:t>>\t"), Ok((">\t", format_hint)));

        let error = nom::Err::Error(nom::error::Error {
            input: "Hello<l:c>",
            code: nom::error::ErrorKind::Tag,
        });
        assert_eq!(layout_hint_parser("Hello<l:c>"), Err(error));

        let error = nom::Err::Error(nom::error::Error {
            input: "<l:c>",
            code: nom::error::ErrorKind::Tag,
        });
        assert_eq!(layout_hint_parser("<<l:c>"), Err(error));
    }

    // Parser for the value of an outlined format hint.
    fn outlined_value_parser(input: &str) -> IResult<&str, &str> {
        alt((tag(TRUE_VALUE_TAG), tag(FALSE_VALUE_TAG)))(input)
    }

    #[test]
    fn test_outlined_value_parser() {
        assert_eq!(outlined_value_parser("t"), Ok(("", "t")));
        assert_eq!(outlined_value_parser("f"), Ok(("", "f")));

        let error = nom::Err::Error(nom::error::Error {
            input: "z",
            code: nom::error::ErrorKind::Tag,
        });
        assert_eq!(outlined_value_parser("z"), Err(error));
    }

    // Parser that matches an outlined format hint.
    fn outlined_hint_parser(input: &str) -> IResult<&str, ParsedValue> {
        let (remainder, result) = tuple((
            format_hint_begin_parser,
            tag(OUTLINED_KEY_TAG),
            format_hint_separator_parser,
            outlined_value_parser,
            format_hint_end_parser,
        ))(input)?;

        Ok((
            remainder,
            ParsedValue::FormatHint {
                key: FormatHintType::Outlined,
                value: result.3.into(),
            },
        ))
    }

    #[test]
    fn test_outlined_hint_parser() {
        let format_hint = ParsedValue::FormatHint {
            key: FormatHintType::Outlined,
            value: "t".into(),
        };
        assert_eq!(outlined_hint_parser("<o:t>>\t"), Ok((">\t", format_hint)));

        let error = nom::Err::Error(nom::error::Error {
            input: "l:c>",
            code: nom::error::ErrorKind::Tag,
        });
        assert_eq!(outlined_hint_parser("<l:c>"), Err(error));
    }

    // Parser for the value of any of the color format hints.
    fn color_value_parser(input: &str) -> IResult<&str, &str> {
        // Due to max tuple size for alt() we must split this into multiple sub parsers.
        alt((
            alt((
                tag(DARK_RED_COLOR_VALUE_TAG),
                tag(BRIGHT_RED_COLOR_VALUE_TAG),
            )),
            alt((
                tag(DARK_ORANGE_COLOR_VALUE_TAG),
                tag(BRIGHT_ORANGE_COLOR_VALUE_TAG),
            )),
            alt((tag(BROWN_COLOR_VALUE_TAG), tag(YELLOW_COLOR_VALUE_TAG))),
            alt((
                tag(DARK_GREEN_COLOR_VALUE_TAG),
                tag(BRIGHT_GREEN_COLOR_VALUE_TAG),
            )),
            alt((
                tag(DARK_BLUE_COLOR_VALUE_TAG),
                tag(BRIGHT_BLUE_COLOR_VALUE_TAG),
            )),
            alt((
                tag(DARK_PURPLE_COLOR_VALUE_TAG),
                tag(BRIGHT_PURPLE_COLOR_VALUE_TAG),
            )),
            alt((
                tag(DARK_CYAN_COLOR_VALUE_TAG),
                tag(BRIGHT_CYAN_COLOR_VALUE_TAG),
            )),
            alt((
                tag(DARK_MAGENTA_COLOR_VALUE_TAG),
                tag(BRIGHT_MAGENTA_COLOR_VALUE_TAG),
            )),
            tag(GOLD_COLOR_VALUE_TAG),
            alt((
                tag(BLACK_COLOR_VALUE_TAG),
                tag(DARK_GREY_COLOR_VALUE_TAG),
                tag(BRIGHT_GREY_COLOR_VALUE_TAG),
                tag(WHITE_COLOR_VALUE_TAG),
            )),
            tag(TRANSPARENT_COLOR_VALUE_TAG),
        ))(input)
    }

    // Parser that matches a foreground color format hint.
    fn foreground_color_hint_parser(input: &str) -> IResult<&str, ParsedValue> {
        let (remainder, result) = tuple((
            format_hint_begin_parser,
            tag(FOREGROUND_COLOR_KEY_TAG),
            format_hint_separator_parser,
            color_value_parser,
            format_hint_end_parser,
        ))(input)?;

        Ok((
            remainder,
            ParsedValue::FormatHint {
                key: FormatHintType::ForegroundColor,
                value: result.3.into(),
            },
        ))
    }

    // Parser that matches a background color format hint.
    fn background_color_hint_parser(input: &str) -> IResult<&str, ParsedValue> {
        let (remainder, result) = tuple((
            format_hint_begin_parser,
            tag(BACKGROUND_COLOR_KEY_TAG),
            format_hint_separator_parser,
            color_value_parser,
            format_hint_end_parser,
        ))(input)?;

        Ok((
            remainder,
            ParsedValue::FormatHint {
                key: FormatHintType::BackgroundColor,
                value: result.3.into(),
            },
        ))
    }

    // Parser that matches an outline color format hint.
    fn outline_color_hint_parser(input: &str) -> IResult<&str, ParsedValue> {
        let (remainder, result) = tuple((
            format_hint_begin_parser,
            tag(OUTLINE_COLOR_KEY_TAG),
            format_hint_separator_parser,
            color_value_parser,
            format_hint_end_parser,
        ))(input)?;

        Ok((
            remainder,
            ParsedValue::FormatHint {
                key: FormatHintType::OutlineColor,
                value: result.3.into(),
            },
        ))
    }

    // Parser that matches any of the possible format hint varieties.
    fn format_hint_parser(input: &str) -> IResult<&str, ParsedValue> {
        alt((
            layout_hint_parser,
            outlined_hint_parser,
            foreground_color_hint_parser,
            background_color_hint_parser,
            outline_color_hint_parser,
        ))(input)
    }

    // The main parse function.
    pub fn parse(input: String) -> Result<Vec<ParsedValue>> {
        let result = many1(alt((
            text_parser,
            newline_parser,
            escaped_chevron_parser,
            format_hint_parser,
        )))(&input);

        Ok(result.map_err(|e| anyhow::format_err!(e.to_string()))?.1)
    }

    #[test]
    fn test_parse() {
        const TEST_STR: &str =
            "<l:t><o:f><fc:Y><bc:k><<<oc:k>Hello, <l:c><o:t><fc:k><oc:R>world<l:t><o:f><fc:Y>!";

        assert_eq!(
            parse(TEST_STR.into()).unwrap(),
            vec![
                ParsedValue::FormatHint {
                    key: FormatHintType::Layout,
                    value: "t".into()
                },
                ParsedValue::FormatHint {
                    key: FormatHintType::Outlined,
                    value: "f".into()
                },
                ParsedValue::FormatHint {
                    key: FormatHintType::ForegroundColor,
                    value: "Y".into()
                },
                ParsedValue::FormatHint {
                    key: FormatHintType::BackgroundColor,
                    value: "k".into()
                },
                ParsedValue::Text("<".into()),
                ParsedValue::FormatHint {
                    key: FormatHintType::OutlineColor,
                    value: "k".into()
                },
                ParsedValue::Text("Hello, ".into()),
                ParsedValue::FormatHint {
                    key: FormatHintType::Layout,
                    value: "c".into()
                },
                ParsedValue::FormatHint {
                    key: FormatHintType::Outlined,
                    value: "t".into()
                },
                ParsedValue::FormatHint {
                    key: FormatHintType::ForegroundColor,
                    value: "k".into()
                },
                ParsedValue::FormatHint {
                    key: FormatHintType::OutlineColor,
                    value: "R".into()
                },
                ParsedValue::Text("world".into()),
                ParsedValue::FormatHint {
                    key: FormatHintType::Layout,
                    value: "t".into()
                },
                ParsedValue::FormatHint {
                    key: FormatHintType::Outlined,
                    value: "f".into()
                },
                ParsedValue::FormatHint {
                    key: FormatHintType::ForegroundColor,
                    value: "Y".into()
                },
                ParsedValue::Text("!".into()),
            ]
        );
    }
}

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

fn main() -> Result<(), String> {
    let sdl2_context = sdl2::init()?;
    let video_subsystem = sdl2_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG);
    let window = video_subsystem
        .window("FVR_ENGINE", 800, 600)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    let _texture_creator = canvas.texture_creator();
    let mut event_pump = sdl2_context.event_pump().map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGB(30, 15, 60));

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        canvas.clear();
        canvas.present();
    }

    Ok(())
}
