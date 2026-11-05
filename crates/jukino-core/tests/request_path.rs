//! Behaviour of [`RequestPath`], a request's location in a collection's folder tree.

use jukino_core::RequestPathError::{Empty, EmptySegment, ForbiddenCharacter, ReservedSegment};
use jukino_core::{RequestPath, RequestPathError};

fn segments_of(input: &str) -> Vec<String> {
    match RequestPath::parse(input) {
        Ok(path) => path.segments().map(str::to_owned).collect(),
        Err(error) => panic!("{input:?} should parse, got: {error}"),
    }
}

#[test]
fn parses_nested_folders_and_the_request_name() {
    assert_eq!(
        segments_of("users/admins/list"),
        ["users", "admins", "list"]
    );
}

#[test]
fn a_single_segment_is_a_request_at_the_collection_root() {
    assert_eq!(segments_of("health"), ["health"]);
}

#[test]
fn display_round_trips_the_parsed_text() {
    for input in ["users/admins/list", "café/ñandú/créer"] {
        let path = RequestPath::parse(input).unwrap();
        assert_eq!(path.to_string(), input);
    }
}

#[test]
fn from_str_agrees_with_parse() {
    assert_eq!("a/b".parse::<RequestPath>(), RequestPath::parse("a/b"));
    assert_eq!("".parse::<RequestPath>(), RequestPath::parse(""));
}

#[test]
fn a_segment_with_interior_spaces_is_accepted() {
    assert_eq!(segments_of("users/get user"), ["users", "get user"]);
}

#[test]
fn empty_input_is_rejected() {
    assert_eq!(RequestPath::parse(""), Err(Empty));
}

#[test]
fn empty_segments_are_rejected() {
    assert_eq!(RequestPath::parse("/a"), Err(EmptySegment { index: 0 }));
    assert_eq!(RequestPath::parse("a/"), Err(EmptySegment { index: 1 }));
    assert_eq!(RequestPath::parse("a//b"), Err(EmptySegment { index: 1 }));
}

#[test]
fn dot_segments_are_rejected() {
    assert_eq!(
        RequestPath::parse("a/./b"),
        Err(ReservedSegment { index: 1 })
    );
    assert_eq!(
        RequestPath::parse("../a"),
        Err(ReservedSegment { index: 0 })
    );
}

#[test]
fn backslash_is_rejected() {
    assert_eq!(
        RequestPath::parse("a\\b"),
        Err(ForbiddenCharacter {
            index: 0,
            character: '\\'
        })
    );
}

#[test]
fn control_characters_are_rejected() {
    for character in ['\n', '\t', '\u{7f}'] {
        let input = format!("a/b{character}c");
        assert_eq!(
            RequestPath::parse(&input),
            Err(ForbiddenCharacter {
                index: 1,
                character
            })
        );
    }
}

#[test]
fn errors_explain_themselves() {
    assert_eq!(Empty.to_string(), "request path is empty");
    assert_eq!(
        EmptySegment { index: 1 }.to_string(),
        "segment 1 of the request path is empty"
    );
    assert_eq!(
        ReservedSegment { index: 0 }.to_string(),
        "segment 0 of the request path is `.` or `..`"
    );
    assert_eq!(
        ForbiddenCharacter {
            index: 0,
            character: '\\'
        }
        .to_string(),
        "segment 0 of the request path contains forbidden character '\\\\'"
    );
}

#[test]
fn errors_are_standard_errors() {
    let error: &dyn core::error::Error = &RequestPathError::Empty;
    assert!(error.source().is_none());
}
