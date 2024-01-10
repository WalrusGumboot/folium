use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::ast::ElementType::*;
use crate::ast::{AbstractElementData, AbstractElementID, ElementType, GlobalState, Slide};
use crate::error::FoliumError;
use crate::style::{PropertyValue, StyleMap, StyleTarget};

use itertools::Itertools;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    /// in source code: token [
    OpeningSlideParen,
    /// in source code: token ]
    ClosingSlideParen,
    /// in source code: token ::
    Definition,
    /// in source code: token :
    ValueAssignment,
    /// in source code: token ,
    ListSeparator,
    /// in source code: token "
    StringDelim,
    /// in source code: token (
    OpeningArgsParen,
    /// in source code: token )
    ClosingArgsParen,
    /// in source code: token {
    OpeningParamsParen,
    /// in source code: token }
    ClosingParamsParen,
    /// in source code: token numbers, string literals, bool
    Value(PropertyValue),
    /// in source code: token all other values
    Ident(&'a str),
}
use Token::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenLocation {
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for TokenLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, col {}", self.line + 1, self.col + 1)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FatToken<'a> {
    token: Token<'a>,
    location: TokenLocation,
}

#[derive(Clone, Debug)]
enum RawToken<'a> {
    AlreadyParsed {
        line_idx: usize,
        col_idx: usize,
        value: Token<'a>,
    },
    NotYetParsed {
        line_idx: usize,
        col_idx: usize,
        value: char,
    },
}

// wat een kankerlelijke functie is mich dat hie
fn split_off_string_delims(mut s: &str) -> Vec<&str> {
    if s == "::" {
        return vec!["::"];
    }
    let mut ret = Vec::new();
    if let Some(new_s) = s.strip_prefix('"') {
        s = new_s;
        ret.push("\"");
    }

    if let Some(new_s) = s.strip_suffix("\",") {
        ret.push(new_s);
        ret.push("\"");
        ret.push(",")
    } else if let Some(new_s) = s.strip_suffix('"') {
        ret.push(new_s);
        ret.push("\"")
    } else if let Some(new_s) = s.strip_suffix(',') {
        ret.push(new_s);
        ret.push(",")
    } else if let Some(new_s) = s.strip_suffix(':') {
        ret.push(new_s);
        ret.push(":")
    } else {
        ret.push(s);
    }

    ret
}

/// Takes an iterator of tokens and returns the defined AbstractElement
fn parse_content_definition<'a, I: std::fmt::Debug + Iterator<Item = FatToken<'a>>>(
    mut iter: I,
    global: &'a GlobalState,
) -> Result<AbstractElementID, FoliumError> {
    let content_name_or_type = iter
        .next()
        .expect("could not parse name of following content item");

    // TODO: check if name isn't already in use

    let (maybe_name, element_type, should_check_opening_paren): (
        Option<String>,
        ElementType,
        bool,
    ) = match content_name_or_type.token {
        Ident(ident_val) => {
            if let Ok(el_type) = ElementType::try_from(ident_val) {
                // the current element should be anonymous! if a Definition token :: follows,
                // we should throw an error
                match iter.next() {
                    Some(FatToken {
                        token: Definition,
                        location,
                    }) => {
                        return Err(FoliumError::UseOfContentTypeName {
                            location,
                            word: el_type.string_rep(),
                        })
                    }
                    Some(FatToken {
                        token: OpeningArgsParen,
                        ..
                    }) => {}
                    Some(FatToken {
                        token: other_token,
                        location,
                    }) => {
                        return Err(FoliumError::ExpectedToken {
                            location,
                            expected: OpeningArgsParen,
                            got: other_token,
                        })
                    }
                    None => {
                        return Err(FoliumError::UnexpectedFileEndWithToken {
                            location: content_name_or_type.location,
                            expected: OpeningArgsParen,
                        })
                    }
                }

                (None, el_type, false)
            } else {
                // We assume, then, that the Ident contains the name for a Definition.
                match iter.next() {
                    Some(FatToken {
                        token: Definition,
                        location,
                    }) => {
                        // We're defining an element, so the type should be a valid element name
                        match iter.next() {
                            None => {
                                return Err(FoliumError::UnexpectedFileEndWithReason {
                                    location,
                                    expected: "a content type",
                                })
                            }
                            Some(FatToken {
                                token: Ident(possibly_el_type),
                                location,
                            }) => {
                                if let Ok(el_type) = ElementType::try_from(possibly_el_type) {
                                    (Some(ident_val.to_string()), el_type, true)
                                } else {
                                    return Err(FoliumError::UnknownType {
                                        location,
                                        offending_token: possibly_el_type,
                                    });
                                }
                            }
                            Some(FatToken {
                                token: other_token,
                                location,
                            }) => {
                                return Err(FoliumError::ExpectedReason {
                                    location,
                                    expected: "a content type",
                                    got: other_token,
                                })
                            }
                        }
                    }
                    Some(FatToken {
                        token: other_token,
                        location,
                    }) => {
                        return Err(FoliumError::ExpectedToken {
                            location,
                            expected: Definition,
                            got: other_token,
                        })
                    }
                    None => {
                        return Err(FoliumError::UnexpectedFileEndWithToken {
                            expected: Definition,
                            location: content_name_or_type.location,
                        })
                    }
                }
            }
        }
        other_token => {
            return Err(FoliumError::ExpectedReason {
                expected: "a content type or name",
                got: other_token,
                location: content_name_or_type.location,
            })
        }
    };

    // Assert that this is followed by an OpeningArgsParen token ( if we
    // haven't done so already
    if should_check_opening_paren {
        match iter.next() {
            Some(FatToken {
                token: OpeningArgsParen,
                ..
            }) => {}
            Some(FatToken {
                token: other_token,
                location,
            }) => {
                return Err(FoliumError::ExpectedToken {
                    location,
                    expected: OpeningArgsParen,
                    got: other_token,
                })
            }
            None => {
                return Err(FoliumError::UnexpectedFileEndWithToken {
                    expected: OpeningArgsParen,
                    location: content_name_or_type.location,
                })
            }
        }
    }

    let mut brackets: u8 = 1;
    let content_tokens = iter
        .take_while(|token| {
            match token.token {
                OpeningArgsParen => brackets += 1,
                ClosingArgsParen => brackets -= 1,
                _ => {}
            };
            brackets > 0
        })
        .collect::<Vec<_>>();

    Ok(match element_type {
        ElNone => global.push_element(AbstractElementData::None, element_type, maybe_name),
        Text => global.push_element(
            AbstractElementData::Text(match content_tokens[0].token {
                Value(PropertyValue::String(ref s)) => s.clone(),
                _ => panic!("text content did not contain text value token"),
            }),
            element_type,
            maybe_name,
        ),
        Code => global.push_element(
            AbstractElementData::Code(match content_tokens[0].token {
                Value(PropertyValue::String(ref s)) => s.clone(),
                _ => panic!("code content did not contain text value token"),
            }),
            element_type,
            maybe_name,
        ),
        Image => global.push_element(
            AbstractElementData::Image(match content_tokens[0].token {
                Value(PropertyValue::String(ref s)) => s.clone().into(),
                _ => panic!("img content did not contain text value token"),
            }),
            element_type,
            maybe_name,
        ),
        Centre => global.push_element(
            AbstractElementData::Centre(
                parse_content_definition(content_tokens.into_iter(), global)
                    .map_err(|err| {
                        eprintln!("{err}");
                        panic!();
                    })
                    .unwrap(),
            ),
            element_type,
            maybe_name,
        ),
        Padding => global.push_element(
            AbstractElementData::Padding(
                parse_content_definition(content_tokens.into_iter(), global)
                    .map_err(|err| {
                        eprintln!("{err}");
                        panic!();
                    })
                    .unwrap(),
            ),
            element_type,
            maybe_name,
        ),
        // Problem: splitting on ListSeparators isn't correct, because contained elements may also have
        // ListSeps in their own definitions
        Row => {
            let children_tokens = split_child_elements(content_tokens.iter().cloned());
            let children_ids = children_tokens
                .into_iter()
                .map(|tokens| {
                    parse_content_definition(tokens.iter().cloned(), global)
                        .map_err(|err| panic!("{err}"))
                        .unwrap()
                })
                .collect();
            global.push_element(
                AbstractElementData::Row(children_ids),
                element_type,
                maybe_name,
            )
        }
        Col => {
            let children_tokens = split_child_elements(content_tokens.iter().cloned());
            let children_ids = children_tokens
                .into_iter()
                .map(|tokens| {
                    parse_content_definition(tokens.iter().cloned(), global)
                        .map_err(|err| panic!("{err}"))
                        .unwrap()
                })
                .collect();
            global.push_element(
                AbstractElementData::Col(children_ids),
                element_type,
                maybe_name,
            )
        }
    })
}

fn split_child_elements<'a, I: std::fmt::Debug + Iterator<Item = FatToken<'a>>>(
    mut iter: I,
) -> Vec<Vec<FatToken<'a>>> {
    let mut children: Vec<Vec<FatToken<'a>>> = Vec::new();

    loop {
        let mut taken_a_bracket = false;
        let mut brackets: usize = 0;

        let token_group = iter
            .by_ref()
            .take_while_inclusive(|token| match token.token {
                OpeningArgsParen => {
                    taken_a_bracket = true;
                    brackets += 1;
                    true
                }
                ClosingArgsParen => {
                    brackets -= 1;
                    brackets != 0 || !taken_a_bracket
                }
                _ => true,
            })
            .collect::<Vec<_>>();

        if token_group.is_empty() {
            break;
        } else {
            if matches!(
                token_group[0],
                FatToken {
                    token: ListSeparator,
                    ..
                }
            ) {
                // TODO: de-uglify this
                children.push(token_group.split_at(1).1.to_vec());
            } else {
                children.push(token_group);
            }
        }
    }

    children
}

pub fn load_from_file<'a, P: AsRef<Path> + 'a>(
    global: &'a GlobalState,
    path: P,
) -> Result<(), FoliumError<'a>> {
    let source = fs::read_to_string(path.as_ref()).expect("could not open file");
    load(global, source)
}

pub fn load(global: &GlobalState, source: String) -> Result<(), FoliumError<'_>> {
    let mut all_characters = source
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.starts_with("//"))
        .flat_map(|(line_idx, line)| {
            line.chars()
                .enumerate()
                .map(|(char_idx, c)| (line_idx, char_idx, c))
                .collect::<Vec<_>>()
        })
        .peekable();

    let mut raw_tokens = Vec::new();

    while let Some((line, col, c)) = all_characters.next() {
        raw_tokens.push(match c {
            '[' => RawToken::AlreadyParsed {
                line_idx: line,
                col_idx: col,
                value: OpeningSlideParen,
            },
            ']' => RawToken::AlreadyParsed {
                line_idx: line,
                col_idx: col,
                value: ClosingSlideParen,
            },
            '(' => RawToken::AlreadyParsed {
                line_idx: line,
                col_idx: col,
                value: OpeningArgsParen,
            },
            ')' => RawToken::AlreadyParsed {
                line_idx: line,
                col_idx: col,
                value: ClosingArgsParen,
            },
            '{' => RawToken::AlreadyParsed {
                line_idx: line,
                col_idx: col,
                value: OpeningParamsParen,
            },
            '}' => RawToken::AlreadyParsed {
                line_idx: line,
                col_idx: col,
                value: ClosingParamsParen,
            },
            '"' => RawToken::AlreadyParsed {
                line_idx: line,
                col_idx: col,
                value: StringDelim,
            },
            ',' => RawToken::AlreadyParsed {
                line_idx: line,
                col_idx: col,
                value: ListSeparator,
            },
            ':' => {
                if all_characters.next_if(|&(_, _, c)| c == ':').is_some() {
                    RawToken::AlreadyParsed {
                        line_idx: line,
                        col_idx: col,
                        value: Definition,
                    }
                } else {
                    RawToken::AlreadyParsed {
                        line_idx: line,
                        col_idx: col,
                        value: ValueAssignment,
                    }
                }
            }
            other => RawToken::NotYetParsed {
                line_idx: line,
                col_idx: col,
                value: other,
            },
        });
    }

    let mut contiguous_tokens: Vec<FatToken> = Vec::new();
    let mut tokens_to_ignore: usize = 0;

    let mut raw_tokens_iter = raw_tokens.into_iter();

    while let Some(next_raw_token) = raw_tokens_iter.next() {
        if tokens_to_ignore > 0 {
            tokens_to_ignore -= 1;
            continue;
        }

        match next_raw_token {
            RawToken::AlreadyParsed {
                value: StringDelim,
                line_idx,
                col_idx,
            } => {
                let string = raw_tokens_iter
                    .clone()
                    .take_while(|elem| {
                        tokens_to_ignore += 1;
                        !matches!(
                            elem,
                            RawToken::AlreadyParsed {
                                value: StringDelim,
                                ..
                            }
                        )
                    })
                    .flat_map(|elem| match elem {
                        RawToken::NotYetParsed { value, .. } => Some(value),
                        RawToken::AlreadyParsed { .. } => None,
                    })
                    .collect::<String>();
                contiguous_tokens.push(FatToken {
                    token: Value(PropertyValue::String(string)),
                    location: TokenLocation {
                        line: line_idx,
                        col: col_idx,
                    },
                });
            }
            RawToken::AlreadyParsed {
                line_idx,
                col_idx,
                value,
            } => {
                contiguous_tokens.push(FatToken {
                    token: value,
                    location: TokenLocation {
                        line: line_idx,
                        col: col_idx,
                    },
                });
            }
            ref token @ RawToken::NotYetParsed {
                line_idx,
                col_idx,
                value,
            } => {
                if value == ' ' {
                    continue;
                }

                // constructing values
                let iter_clone = raw_tokens_iter.clone();
                let new_iterator = &[token].into_iter().chain(iter_clone.as_ref());

                let working_value: String = new_iterator
                    .clone()
                    .take_while(|elem| {
                        let retval = !matches!(
                            elem,
                            RawToken::AlreadyParsed { .. }
                                | RawToken::NotYetParsed { value: ' ', .. }
                                | RawToken::NotYetParsed { value: ',', .. }
                        );
                        if retval {
                            tokens_to_ignore += 1;
                        }
                        retval
                    })
                    .flat_map(|elem| match elem {
                        RawToken::NotYetParsed { value, .. } => Some(value),
                        RawToken::AlreadyParsed { .. } => unreachable!(),
                    })
                    .collect();

                tokens_to_ignore = tokens_to_ignore.saturating_sub(1);

                if let Ok(number) = working_value.parse::<u32>() {
                    contiguous_tokens.push(FatToken {
                        location: TokenLocation {
                            line: line_idx,
                            col: col_idx,
                        },
                        token: Value(PropertyValue::Number(number)),
                    });
                } else if let Ok(boolean) = working_value.parse::<bool>() {
                    contiguous_tokens.push(FatToken {
                        location: TokenLocation {
                            line: line_idx,
                            col: col_idx,
                        },
                        token: Value(PropertyValue::Boolean(boolean)),
                    });
                } else {
                    let token = 
                    if working_value.starts_with('#')
                        && working_value.len() == 7
                        && working_value.chars().skip(1).all(|c| c.is_ascii_hexdigit())
                    {
                        // parseable as colour

                        let colour = working_value.as_str();
                        let r = u8::from_str_radix(&colour[1..3], 16).unwrap();
                        let g = u8::from_str_radix(&colour[3..5], 16).unwrap();
                        let b = u8::from_str_radix(&colour[5..7], 16).unwrap();

                        Value(PropertyValue::Colour(r, g, b))
                    } else {
                        // TODO: don't leak memory
                        Ident(working_value.leak())
                    };

                    contiguous_tokens.push(FatToken {
                        location: TokenLocation {
                            line: line_idx,
                            col: col_idx,
                        },
                        token
                    });
                }
            }
        }
    }

    // group tokens by slide
    let mut grouped_tokens: Vec<Vec<FatToken>> = Vec::new();
    let mut current_slide_tokens: Vec<FatToken> = Vec::new();

    for fat_token in contiguous_tokens {
        match fat_token {
            FatToken {
                token: OpeningSlideParen,
                ..
            } => {}
            FatToken {
                token: ClosingSlideParen,
                ..
            } => {
                grouped_tokens.push(current_slide_tokens.clone());
                current_slide_tokens.clear();
            }
            other => current_slide_tokens.push(other),
        }
    }

    for slide_tokens in grouped_tokens {
        let mut iter = slide_tokens.into_iter();
        let content_root_id = parse_content_definition(&mut iter, global)
            .map_err(|err| {
                eprintln!("{err}");
                panic!()
            })
            .unwrap();

        let remaining_style_tokens = iter.collect::<Vec<_>>();

        let style_map: StyleMap = if !remaining_style_tokens.is_empty() {
            let individual_styles = remaining_style_tokens
                .split(|token| token.token == ClosingParamsParen)
                .filter(|slice| !slice.is_empty());
            let mut style_map = StyleMap::new();

            for individual_style in individual_styles {
                let target = match &individual_style[0] {
                    &FatToken {
                        token: Ident(ident_val),
                        ..
                    } => {
                        if let Ok(el_type) = ElementType::try_from(ident_val) {
                            StyleTarget::Anonymous(el_type)
                        } else if ident_val == "slide" {
                            StyleTarget::Slide
                        } else {
                            StyleTarget::Named(ident_val.to_owned())
                        }
                    }
                    FatToken {
                        token: other_token,
                        location,
                    } => {
                        return Err(FoliumError::ExpectedReason {
                            expected: "a style target identifier",
                            location: *location,
                            got: other_token.clone(),
                        })
                    }
                };

                let properties: HashMap<String, PropertyValue> = individual_style[2..]
                    .chunks(4) // we use chunks instead of chunks_exact because it doesn't enfore a comma after the last element
                    .map(|slice| &slice[0..3])
                    .map(|def| {
                        assert_eq!(def[1].token, Token::ValueAssignment);
                        (
                            (match &def[0] {
                                FatToken {
                                    token: Ident(s), ..
                                } => Ok(s.to_string()),
                                FatToken {
                                    token: other_token,
                                    location,
                                } => Err(FoliumError::ExpectedReason {
                                    location: *location,
                                    expected: "a style directive",
                                    got: other_token.clone(),
                                }),
                            })
                            .map_err(|err| panic!("{err}"))
                            .unwrap(),
                            match &def[2] {
                                FatToken {
                                    token: Value(pv), ..
                                } => Ok(pv),
                                FatToken {
                                    token: other_token,
                                    location,
                                } => Err(FoliumError::ExpectedReason {
                                    location: *location,
                                    expected: "a parameter value",
                                    got: other_token.clone(),
                                }),
                            }
                            .map_err(|err| panic!("{err}"))
                            .unwrap()
                            .clone(),
                        )
                    })
                    .collect();

                style_map.add_style(target, properties);
            }

            // make sure that properties like height and width are present if the user hasn't overridden them
            style_map.fill_in(StyleMap::default());

            style_map
        } else {
            StyleMap::default()
        };

        let slide = Slide::new(global, content_root_id, style_map);
        global.push_slide(slide);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_slide() {
        let global = GlobalState::new();
        let source = String::from("[ none() ]");
        assert_eq!(Ok(()), load(&global, source));
        let none_el = global.get_element_by_id(AbstractElementID(1)).unwrap();
        assert_eq!(none_el.data(), &AbstractElementData::None);
        assert_eq!(*none_el.name(), None);
    }

    #[test]
    fn named_none_slide() {
        let global = GlobalState::new();
        let source = String::from("[ joop :: none() ]");
        assert_eq!(Ok(()), load(&global, source));
        let none_el = global.get_element_by_id(AbstractElementID(1)).unwrap();
        assert_eq!(none_el.data(), &AbstractElementData::None);
        assert_eq!(*none_el.name(), Some(String::from("joop")));
    }

    #[test]
    fn text_slide() {
        let global = GlobalState::new();
        let source = String::from(r#"[ text("jakob") ]"#);
        assert_eq!(Ok(()), load(&global, source));
        let text_el = global.get_element_by_id(AbstractElementID(1)).unwrap();
        assert_eq!(
            text_el.data(),
            &AbstractElementData::Text(String::from("jakob"))
        );
    }

    #[test]
    fn named_text_slide() {
        let global = GlobalState::new();
        let source = String::from(r#"[ joop :: text("jakob") ]"#);
        assert_eq!(Ok(()), load(&global, source));
        let text_el = global.get_element_by_id(AbstractElementID(1)).unwrap();
        assert_eq!(
            text_el.data(),
            &AbstractElementData::Text(String::from("jakob"))
        );
    }

    #[test]
    fn named_text_slide_with_space() {
        let global = GlobalState::new();
        let source = String::from(r#"[ joop :: text("jakob en zonen") ]"#);
        assert_eq!(Ok(()), load(&global, source));
        let text_el = global.get_element_by_id(AbstractElementID(1)).unwrap();
        assert_eq!(
            text_el.data(),
            &AbstractElementData::Text(String::from("jakob en zonen"))
        );
    }

    #[test]
    fn styled_slide() {
        let global = GlobalState::new();
        let source = String::from(r#"[ padding ( text ("joop") ) padding { amount: 10, } ]"#);
        assert_eq!(Ok(()), load(&global, source));

        let slides = global.slides.borrow();
        let slide = &slides[0];

        let padding_style = slide
            .style_map()
            .styles_for_target(StyleTarget::Anonymous(Padding))
            .unwrap();
        let padding_amount = padding_style.get(&String::from("amount")).unwrap();
        assert_eq!(padding_amount, &PropertyValue::Number(10))
    }

    #[test]
    fn partial_style_override() {
        let global = GlobalState::new();
        let source = String::from(r#"[ none () slide { height: 500 } ]"#);
        assert_eq!(Ok(()), load(&global, source));

        let slides = global.slides.borrow();
        let slide = &slides[0];

        let slide_style = slide
            .style_map()
            .styles_for_target(StyleTarget::Slide)
            .unwrap();
        let height = slide_style.get(&String::from("height")).unwrap();
        let width = slide_style.get(&String::from("width")).unwrap();
        assert_eq!(height, &PropertyValue::Number(500));
        assert_eq!(width, &PropertyValue::Number(1920));
    }

    #[test]
    fn col_in_row() {
        let global = GlobalState::new();
        let source = String::from(
            r#"[ row ( text("joop"), col ( text("in kolom"), text("in kolom 2") ) ) ]"#,
        );
        assert_eq!(Ok(()), load(&global, source));

        println!("{}", global);

        let row = global.get_element_by_id(AbstractElementID(5)).unwrap();
        let data = match row.data() {
            AbstractElementData::Row(val) => val,
            _ => panic!(),
        };
        assert_eq!(data.len(), 2);
    }
}
