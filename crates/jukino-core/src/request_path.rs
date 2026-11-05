//! A request's location in a collection's folder tree.

use core::fmt;
use core::str::FromStr;

/// The location of a request inside a collection: folder names followed by the
/// request's name, separated by `/`, as in `users/admins/list`.
///
/// Every segment is non-empty, contains no `\` and no control character, and is
/// neither `.` nor `..`. Build one with [`RequestPath::parse`] or [`str::parse`];
/// its [`Display`](fmt::Display) output is the text it was parsed from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestPath {
    segments: Vec<String>,
}

impl RequestPath {
    /// Parses `input` into a request path.
    ///
    /// # Errors
    ///
    /// Returns [`RequestPathError::Empty`] when `input` is empty, otherwise the error
    /// for the first segment, from the left, that breaks an invariant.
    pub fn parse(input: &str) -> Result<Self, RequestPathError> {
        if input.is_empty() {
            return Err(RequestPathError::Empty);
        }
        let segments = input
            .split('/')
            .enumerate()
            .map(|(index, segment)| check_segment(index, segment).map(|()| segment.to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { segments })
    }

    /// The folder names, outermost first, followed by the request's name.
    pub fn segments(&self) -> impl Iterator<Item = &str> + '_ {
        self.segments.iter().map(String::as_str)
    }
}

fn check_segment(index: usize, segment: &str) -> Result<(), RequestPathError> {
    if segment.is_empty() {
        return Err(RequestPathError::EmptySegment { index });
    }
    if segment == "." || segment == ".." {
        return Err(RequestPathError::ReservedSegment { index });
    }
    match segment.chars().find(|&c| c == '\\' || c.is_control()) {
        Some(character) => Err(RequestPathError::ForbiddenCharacter { index, character }),
        None => Ok(()),
    }
}

impl fmt::Display for RequestPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.segments.join("/"))
    }
}

impl FromStr for RequestPath {
    type Err = RequestPathError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

/// Why a text is not a valid [`RequestPath`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestPathError {
    /// The text is empty.
    Empty,
    /// A segment is empty, as with `a//b` or a leading or trailing `/`.
    EmptySegment {
        /// Position of the segment, counting from 0.
        index: usize,
    },
    /// A segment is `.` or `..`, which would point outside the folder tree.
    ReservedSegment {
        /// Position of the segment, counting from 0.
        index: usize,
    },
    /// A segment contains `\` or a control character.
    ForbiddenCharacter {
        /// Position of the segment, counting from 0.
        index: usize,
        /// The first forbidden character in the segment.
        character: char,
    },
}

impl fmt::Display for RequestPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("request path is empty"),
            Self::EmptySegment { index } => {
                write!(formatter, "segment {index} of the request path is empty")
            }
            Self::ReservedSegment { index } => {
                write!(
                    formatter,
                    "segment {index} of the request path is `.` or `..`"
                )
            }
            Self::ForbiddenCharacter { index, character } => write!(
                formatter,
                "segment {index} of the request path contains forbidden character {character:?}"
            ),
        }
    }
}

impl core::error::Error for RequestPathError {}
