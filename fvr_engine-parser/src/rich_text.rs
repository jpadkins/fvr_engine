//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1};
use nom::multi::many1;
use nom::sequence::tuple;
use nom::IResult;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

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
const STYLE_KEY_TAG: &str = "st";
const SIZE_KEY_TAG: &str = "si";
const OUTLINED_KEY_TAG: &str = "o";
const FOREGROUND_COLOR_KEY_TAG: &str = "fc";
const BACKGROUND_COLOR_KEY_TAG: &str = "bc";
const OUTLINE_COLOR_KEY_TAG: &str = "oc";

// Tags for the possible layout values.
const CENTER_LAYOUT_VALUE_TAG: &str = "c";
const FLOOR_LAYOUT_VALUE_TAG: &str = "f";
const TEXT_LAYOUT_VALUE_TAG: &str = "t";

// Tags for the possible style values.
const REGULAR_STYLE_VALUE_TAG: &str = "r";
const BOLD_STYLE_VALUE_TAG: &str = "b";
const ITALIC_STYLE_VALUE_TAG: &str = "i";
const BOLD_ITALIC_STYLE_VALUE_TAG: &str = "bi";

// Tags for the possible size values.
const SMALL_SIZE_VALUE_TAG: &str = "s";
const NORMAL_SIZE_VALUE_TAG: &str = "n";
const BIG_SIZE_VALUE_TAG: &str = "b";
const GIANT_SIZE_VALUE_TAG: &str = "g";

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

//-------------------------------------------------------------------------------------------------
// Enum of possible types of format hints.
//-------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq)]
pub enum RichTextHintType {
    Layout,
    Style,
    Size,
    Outlined,
    ForegroundColor,
    BackgroundColor,
    OutlineColor,
}

impl RichTextHintType {
    pub fn to_key_value(&self) -> &'static str {
        match self {
            RichTextHintType::Layout => LAYOUT_KEY_TAG,
            RichTextHintType::Style => STYLE_KEY_TAG,
            RichTextHintType::Size => SIZE_KEY_TAG,
            RichTextHintType::Outlined => OUTLINED_KEY_TAG,
            RichTextHintType::ForegroundColor => FOREGROUND_COLOR_KEY_TAG,
            RichTextHintType::BackgroundColor => BACKGROUND_COLOR_KEY_TAG,
            RichTextHintType::OutlineColor => OUTLINE_COLOR_KEY_TAG,
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Enum of possible parsed values, which can either be text, a newline, or a format hint.
//-------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq)]
pub enum RichTextValue {
    Text(String),
    Newline,
    FormatHint { key: RichTextHintType, value: String },
}

//-------------------------------------------------------------------------------------------------
// Parser for a string of text that does not contain a newline, escaped left chevron, or format hint.
//-------------------------------------------------------------------------------------------------
fn text_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, result) = take_till1(|c: char| c == NEWLINE || c == LEFT_CHEVRON)(input)?;

    Ok((remainder, RichTextValue::Text(result.into())))
}

//-------------------------------------------------------------------------------------------------
// Parser for a single newline character.
//-------------------------------------------------------------------------------------------------
fn newline_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, _) = tag(NEWLINE_TAG)(input)?;

    Ok((remainder, RichTextValue::Newline))
}

//-------------------------------------------------------------------------------------------------
// Parser for double (escaped) left chevron, which translates to a single left chevron.
//-------------------------------------------------------------------------------------------------
fn escaped_chevron_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, _) = tag(DOUBLE_LEFT_CHEVRON_TAG)(input)?;

    Ok((remainder, RichTextValue::Text(LEFT_CHEVRON_TAG.into())))
}

//-------------------------------------------------------------------------------------------------
// Parser for a single left chevron, which designates the start of a format hint.
//-------------------------------------------------------------------------------------------------
fn format_hint_begin_parser(input: &str) -> IResult<&str, &str> {
    tag(LEFT_CHEVRON_TAG)(input)
}

//-------------------------------------------------------------------------------------------------
// Parser for the format hint key/value separator colon.
//-------------------------------------------------------------------------------------------------
fn format_hint_separator_parser(input: &str) -> IResult<&str, &str> {
    tag(COLON_TAG)(input)
}

//-------------------------------------------------------------------------------------------------
// Parser for a single right chevron, which designates the end of a format hint.
//-------------------------------------------------------------------------------------------------
fn format_hint_end_parser(input: &str) -> IResult<&str, &str> {
    tag(RIGHT_CHEVRON_TAG)(input)
}

//-------------------------------------------------------------------------------------------------
// Parses the value of a layout format hint.
//-------------------------------------------------------------------------------------------------
fn layout_value_parser(input: &str) -> IResult<&str, &str> {
    alt((tag(CENTER_LAYOUT_VALUE_TAG), tag(FLOOR_LAYOUT_VALUE_TAG), tag(TEXT_LAYOUT_VALUE_TAG)))(
        input,
    )
}

//-------------------------------------------------------------------------------------------------
// Parser that matches a layout format hint.
//-------------------------------------------------------------------------------------------------
fn layout_hint_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, result) = tuple((
        format_hint_begin_parser,
        tag(LAYOUT_KEY_TAG),
        format_hint_separator_parser,
        layout_value_parser,
        format_hint_end_parser,
    ))(input)?;

    Ok((
        remainder,
        RichTextValue::FormatHint { key: RichTextHintType::Layout, value: result.3.into() },
    ))
}

//-------------------------------------------------------------------------------------------------
// Parser that matches the value of a style format hint.
//-------------------------------------------------------------------------------------------------
fn style_value_parser(input: &str) -> IResult<&str, &str> {
    // Note: We must attempt to parse the bold italic tag "bi" before the bold tag "b".
    alt((
        tag(REGULAR_STYLE_VALUE_TAG),
        tag(BOLD_ITALIC_STYLE_VALUE_TAG),
        tag(BOLD_STYLE_VALUE_TAG),
        tag(ITALIC_STYLE_VALUE_TAG),
    ))(input)
}

//-------------------------------------------------------------------------------------------------
// Parser that matches a style format hint.
//-------------------------------------------------------------------------------------------------
fn style_hint_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, result) = tuple((
        format_hint_begin_parser,
        tag(STYLE_KEY_TAG),
        format_hint_separator_parser,
        style_value_parser,
        format_hint_end_parser,
    ))(input)?;

    Ok((
        remainder,
        RichTextValue::FormatHint { key: RichTextHintType::Style, value: result.3.into() },
    ))
}

//-------------------------------------------------------------------------------------------------
// Parser that matches the value of a size format hint.
//-------------------------------------------------------------------------------------------------
fn size_value_parser(input: &str) -> IResult<&str, &str> {
    // Note: We must attempt to parse the bold italic tag "bi" before the bold tag "b".
    alt((
        tag(SMALL_SIZE_VALUE_TAG),
        tag(NORMAL_SIZE_VALUE_TAG),
        tag(BIG_SIZE_VALUE_TAG),
        tag(GIANT_SIZE_VALUE_TAG),
    ))(input)
}

//-------------------------------------------------------------------------------------------------
// Parser that matches a size format hint.
//-------------------------------------------------------------------------------------------------
fn size_hint_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, result) = tuple((
        format_hint_begin_parser,
        tag(SIZE_KEY_TAG),
        format_hint_separator_parser,
        size_value_parser,
        format_hint_end_parser,
    ))(input)?;

    Ok((
        remainder,
        RichTextValue::FormatHint { key: RichTextHintType::Size, value: result.3.into() },
    ))
}

//-------------------------------------------------------------------------------------------------
// Parser for the value of an outlined format hint.
//-------------------------------------------------------------------------------------------------
fn outlined_value_parser(input: &str) -> IResult<&str, &str> {
    alt((tag(TRUE_VALUE_TAG), tag(FALSE_VALUE_TAG)))(input)
}

//-------------------------------------------------------------------------------------------------
// Parser that matches an outlined format hint.
//-------------------------------------------------------------------------------------------------
fn outlined_hint_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, result) = tuple((
        format_hint_begin_parser,
        tag(OUTLINED_KEY_TAG),
        format_hint_separator_parser,
        outlined_value_parser,
        format_hint_end_parser,
    ))(input)?;

    Ok((
        remainder,
        RichTextValue::FormatHint { key: RichTextHintType::Outlined, value: result.3.into() },
    ))
}

//-------------------------------------------------------------------------------------------------
// Parser for the value of any of the color format hints.
//-------------------------------------------------------------------------------------------------
fn color_value_parser(input: &str) -> IResult<&str, &str> {
    // Due to max tuple size for alt() we must split this into multiple sub parsers.
    alt((
        alt((tag(DARK_RED_COLOR_VALUE_TAG), tag(BRIGHT_RED_COLOR_VALUE_TAG))),
        alt((tag(DARK_ORANGE_COLOR_VALUE_TAG), tag(BRIGHT_ORANGE_COLOR_VALUE_TAG))),
        alt((tag(BROWN_COLOR_VALUE_TAG), tag(YELLOW_COLOR_VALUE_TAG))),
        alt((tag(DARK_GREEN_COLOR_VALUE_TAG), tag(BRIGHT_GREEN_COLOR_VALUE_TAG))),
        alt((tag(DARK_BLUE_COLOR_VALUE_TAG), tag(BRIGHT_BLUE_COLOR_VALUE_TAG))),
        alt((tag(DARK_PURPLE_COLOR_VALUE_TAG), tag(BRIGHT_PURPLE_COLOR_VALUE_TAG))),
        alt((tag(DARK_CYAN_COLOR_VALUE_TAG), tag(BRIGHT_CYAN_COLOR_VALUE_TAG))),
        alt((tag(DARK_MAGENTA_COLOR_VALUE_TAG), tag(BRIGHT_MAGENTA_COLOR_VALUE_TAG))),
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

//-------------------------------------------------------------------------------------------------
// Parser that matches a foreground color format hint.
//-------------------------------------------------------------------------------------------------
fn foreground_color_hint_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, result) = tuple((
        format_hint_begin_parser,
        tag(FOREGROUND_COLOR_KEY_TAG),
        format_hint_separator_parser,
        color_value_parser,
        format_hint_end_parser,
    ))(input)?;

    Ok((
        remainder,
        RichTextValue::FormatHint {
            key: RichTextHintType::ForegroundColor,
            value: result.3.into(),
        },
    ))
}

//-------------------------------------------------------------------------------------------------
// Parser that matches a background color format hint.
//-------------------------------------------------------------------------------------------------
fn background_color_hint_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, result) = tuple((
        format_hint_begin_parser,
        tag(BACKGROUND_COLOR_KEY_TAG),
        format_hint_separator_parser,
        color_value_parser,
        format_hint_end_parser,
    ))(input)?;

    Ok((
        remainder,
        RichTextValue::FormatHint {
            key: RichTextHintType::BackgroundColor,
            value: result.3.into(),
        },
    ))
}

//-------------------------------------------------------------------------------------------------
// Parser that matches an outline color format hint.
//-------------------------------------------------------------------------------------------------
fn outline_color_hint_parser(input: &str) -> IResult<&str, RichTextValue> {
    let (remainder, result) = tuple((
        format_hint_begin_parser,
        tag(OUTLINE_COLOR_KEY_TAG),
        format_hint_separator_parser,
        color_value_parser,
        format_hint_end_parser,
    ))(input)?;

    Ok((
        remainder,
        RichTextValue::FormatHint { key: RichTextHintType::OutlineColor, value: result.3.into() },
    ))
}

//-------------------------------------------------------------------------------------------------
// Parser that matches any of the possible format hint varieties.
//-------------------------------------------------------------------------------------------------
fn format_hint_parser(input: &str) -> IResult<&str, RichTextValue> {
    alt((
        layout_hint_parser,
        style_hint_parser,
        size_hint_parser,
        outlined_hint_parser,
        foreground_color_hint_parser,
        background_color_hint_parser,
        outline_color_hint_parser,
    ))(input)
}

//-------------------------------------------------------------------------------------------------
// The main parse function.
//-------------------------------------------------------------------------------------------------
pub fn parse_rich_text<S: AsRef<str>>(input: S) -> Result<Vec<RichTextValue>> {
    let result =
        many1(alt((text_parser, newline_parser, escaped_chevron_parser, format_hint_parser)))(
            input.as_ref(),
        );

    Ok(result.map_err(|e| anyhow::format_err!(e.to_string()))?.1)
}

//-------------------------------------------------------------------------------------------------
// Tests.
//-------------------------------------------------------------------------------------------------

#[test]
fn test_text_parser() {
    assert_eq!(text_parser("abcdefg"), Ok(("", RichTextValue::Text("abcdefg".into()))));
    assert_eq!(text_parser("abc<defg"), Ok(("<defg", RichTextValue::Text("abc".into()))));
    assert_eq!(text_parser("abc\ndefg"), Ok(("\ndefg", RichTextValue::Text("abc".into()))));

    let error = nom::Err::Error(nom::error::Error {
        input: "<abcdefg",
        code: nom::error::ErrorKind::TakeTill1,
    });
    assert_eq!(text_parser("<abcdefg"), Err(error));
}

#[test]
fn test_newline_parser() {
    assert_eq!(newline_parser("\nabc"), Ok(("abc", RichTextValue::Newline)));

    let error =
        nom::Err::Error(nom::error::Error { input: "abc\n", code: nom::error::ErrorKind::Tag });
    assert_eq!(newline_parser("abc\n"), Err(error));
}

#[test]
fn test_escaped_chevron_parser() {
    assert_eq!(escaped_chevron_parser("<<abcd"), Ok(("abcd", RichTextValue::Text("<".into()))));

    let error =
        nom::Err::Error(nom::error::Error { input: "<abcd", code: nom::error::ErrorKind::Tag });
    assert_eq!(escaped_chevron_parser("<abcd"), Err(error));
}

#[test]
fn test_layout_value_parser() {
    assert_eq!(layout_value_parser("c"), Ok(("", "c")));
    assert_eq!(layout_value_parser("f"), Ok(("", "f")));
    assert_eq!(layout_value_parser("t"), Ok(("", "t")));

    let error =
        nom::Err::Error(nom::error::Error { input: "z", code: nom::error::ErrorKind::Tag });
    assert_eq!(layout_value_parser("z"), Err(error));
}

#[test]
fn test_layout_hint_parser() {
    let format_hint =
        RichTextValue::FormatHint { key: RichTextHintType::Layout, value: "c".into() };
    assert_eq!(layout_hint_parser("<l:c>"), Ok(("", format_hint)));

    let format_hint =
        RichTextValue::FormatHint { key: RichTextHintType::Layout, value: "f".into() };
    assert_eq!(layout_hint_parser("<l:f>Hello"), Ok(("Hello", format_hint)));

    let format_hint =
        RichTextValue::FormatHint { key: RichTextHintType::Layout, value: "t".into() };
    assert_eq!(layout_hint_parser("<l:t>>\t"), Ok((">\t", format_hint)));

    let error = nom::Err::Error(nom::error::Error {
        input: "Hello<l:c>",
        code: nom::error::ErrorKind::Tag,
    });
    assert_eq!(layout_hint_parser("Hello<l:c>"), Err(error));

    let error =
        nom::Err::Error(nom::error::Error { input: "<l:c>", code: nom::error::ErrorKind::Tag });
    assert_eq!(layout_hint_parser("<<l:c>"), Err(error));
}

#[test]
fn test_style_value_parser() {
    assert_eq!(style_value_parser("r"), Ok(("", "r")));
    assert_eq!(style_value_parser("b"), Ok(("", "b")));
    assert_eq!(style_value_parser("i"), Ok(("", "i")));
    assert_eq!(style_value_parser("bi"), Ok(("", "bi")));

    let error =
        nom::Err::Error(nom::error::Error { input: "z", code: nom::error::ErrorKind::Tag });
    assert_eq!(style_value_parser("z"), Err(error));
}

#[test]
fn test_style_hint_parser() {
    let format_hint =
        RichTextValue::FormatHint { key: RichTextHintType::Style, value: "r".into() };
    assert_eq!(style_hint_parser("<st:r>"), Ok(("", format_hint)));

    let format_hint =
        RichTextValue::FormatHint { key: RichTextHintType::Style, value: "b".into() };
    assert_eq!(style_hint_parser("<st:b>"), Ok(("", format_hint)));

    let format_hint =
        RichTextValue::FormatHint { key: RichTextHintType::Style, value: "i".into() };
    assert_eq!(style_hint_parser("<st:i>"), Ok(("", format_hint)));

    let format_hint =
        RichTextValue::FormatHint { key: RichTextHintType::Style, value: "bi".into() };
    assert_eq!(style_hint_parser("<st:bi>"), Ok(("", format_hint)));

    let error = nom::Err::Error(nom::error::Error {
        input: "Hello<st:r>",
        code: nom::error::ErrorKind::Tag,
    });
    assert_eq!(style_hint_parser("Hello<st:r>"), Err(error));

    let error =
        nom::Err::Error(nom::error::Error { input: "<st:b>", code: nom::error::ErrorKind::Tag });
    assert_eq!(style_hint_parser("<<st:b>"), Err(error));
}

#[test]
fn test_size_value_parser() {
    assert_eq!(size_value_parser("s"), Ok(("", "s")));
    assert_eq!(size_value_parser("n"), Ok(("", "n")));
    assert_eq!(size_value_parser("b"), Ok(("", "b")));
    assert_eq!(size_value_parser("g"), Ok(("", "g")));

    let error =
        nom::Err::Error(nom::error::Error { input: "z", code: nom::error::ErrorKind::Tag });
    assert_eq!(size_value_parser("z"), Err(error));
}

#[test]
fn test_size_hint_parser() {
    let format_hint = RichTextValue::FormatHint { key: RichTextHintType::Size, value: "s".into() };
    assert_eq!(size_hint_parser("<si:s>"), Ok(("", format_hint)));

    let format_hint = RichTextValue::FormatHint { key: RichTextHintType::Size, value: "n".into() };
    assert_eq!(size_hint_parser("<si:n>"), Ok(("", format_hint)));

    let format_hint = RichTextValue::FormatHint { key: RichTextHintType::Size, value: "b".into() };
    assert_eq!(size_hint_parser("<si:b>"), Ok(("", format_hint)));

    let format_hint = RichTextValue::FormatHint { key: RichTextHintType::Size, value: "g".into() };
    assert_eq!(size_hint_parser("<si:g>"), Ok(("", format_hint)));

    let error = nom::Err::Error(nom::error::Error {
        input: "Hello<si:n>",
        code: nom::error::ErrorKind::Tag,
    });
    assert_eq!(size_hint_parser("Hello<si:n>"), Err(error));

    let error =
        nom::Err::Error(nom::error::Error { input: "<si:s>", code: nom::error::ErrorKind::Tag });
    assert_eq!(size_hint_parser("<<si:s>"), Err(error));
}

#[test]
fn test_outlined_value_parser() {
    assert_eq!(outlined_value_parser("t"), Ok(("", "t")));
    assert_eq!(outlined_value_parser("f"), Ok(("", "f")));

    let error =
        nom::Err::Error(nom::error::Error { input: "z", code: nom::error::ErrorKind::Tag });
    assert_eq!(outlined_value_parser("z"), Err(error));
}

#[test]
fn test_outlined_hint_parser() {
    let format_hint =
        RichTextValue::FormatHint { key: RichTextHintType::Outlined, value: "t".into() };
    assert_eq!(outlined_hint_parser("<o:t>>\t"), Ok((">\t", format_hint)));

    let error =
        nom::Err::Error(nom::error::Error { input: "l:c>", code: nom::error::ErrorKind::Tag });
    assert_eq!(outlined_hint_parser("<l:c>"), Err(error));
}

#[test]
fn test_parse_rich_text() {
    const TEST_STR: &str =
        "<l:t><si:n><st:bi><o:f><fc:Y><bc:k><<<oc:k>Hello, <l:c><o:t><fc:k><oc:R>world<l:t><o:f><fc:Y>!";

    assert_eq!(
        parse_rich_text(TEST_STR).unwrap(),
        vec![
            RichTextValue::FormatHint { key: RichTextHintType::Layout, value: "t".into() },
            RichTextValue::FormatHint { key: RichTextHintType::Size, value: "n".into() },
            RichTextValue::FormatHint { key: RichTextHintType::Style, value: "bi".into() },
            RichTextValue::FormatHint { key: RichTextHintType::Outlined, value: "f".into() },
            RichTextValue::FormatHint {
                key: RichTextHintType::ForegroundColor,
                value: "Y".into()
            },
            RichTextValue::FormatHint {
                key: RichTextHintType::BackgroundColor,
                value: "k".into()
            },
            RichTextValue::Text("<".into()),
            RichTextValue::FormatHint { key: RichTextHintType::OutlineColor, value: "k".into() },
            RichTextValue::Text("Hello, ".into()),
            RichTextValue::FormatHint { key: RichTextHintType::Layout, value: "c".into() },
            RichTextValue::FormatHint { key: RichTextHintType::Outlined, value: "t".into() },
            RichTextValue::FormatHint {
                key: RichTextHintType::ForegroundColor,
                value: "k".into()
            },
            RichTextValue::FormatHint { key: RichTextHintType::OutlineColor, value: "R".into() },
            RichTextValue::Text("world".into()),
            RichTextValue::FormatHint { key: RichTextHintType::Layout, value: "t".into() },
            RichTextValue::FormatHint { key: RichTextHintType::Outlined, value: "f".into() },
            RichTextValue::FormatHint {
                key: RichTextHintType::ForegroundColor,
                value: "Y".into()
            },
            RichTextValue::Text("!".into()),
        ]
    );
}
