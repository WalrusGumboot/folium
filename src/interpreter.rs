use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::ast::ElementType::*;
use crate::ast::{
    AbstractElementData, AbstractElementID, ElementType, GlobalState, PropertyValue, Slide,
};
use crate::error::FoliumError;
use crate::style::{StyleMap, StyleTarget};

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

#[derive(Clone, Debug)]
enum RawToken<'a> {
    AlreadyParseable(Token<'a>),
    NotYetParsed(&'a str),
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
fn parse_content_definition<'a, I: std::fmt::Debug + Iterator<Item = Token<'a>>>(
    mut iter: I,
    global: &'a GlobalState,
) -> Result<AbstractElementID, FoliumError> {
    let content_name_or_type = iter
        .next()
        .expect("could not parse name of following content item");

    // TODO: check if name isn't already in use

    let (maybe_name, element_type, should_check_opening_paren): (Option<String>, ElementType, bool) = match content_name_or_type {
        Ident(ident_val) => {
            if let Ok(el_type) = ElementType::try_from(ident_val) {
                // the current element should be anonymous! if a Definition token :: follows,
                // we should throw an error
                match iter.next() {
                    Some(Definition) => {
                        return Err(FoliumError::UseOfContentTypeName {
                            word: el_type.string_rep(),
                        })
                    }
                    Some(OpeningArgsParen) => {}
                    Some(other_token) => {
                        return Err(FoliumError::ExpectedToken {
                            expected: OpeningArgsParen,
                            got: other_token,
                        })
                    }
                    None => {
                        return Err(FoliumError::UnexpectedFileEndWithToken {
                            expected: OpeningArgsParen,
                        })
                    }
                }

                (None, el_type, false)
            } else {
                // We assume, then, that the Ident contains the name for a Definition.
                match iter.next() {
                    Some(Definition) => {
                        // We're defining an element, so the type should be a valid element name
                        match iter.next() {
                            None => {
                                return Err(FoliumError::UnexpectedFileEndWithReason {
                                    expected: "a content type",
                                })
                            }
                            Some(Ident(possibly_el_type)) => {
                                if let Ok(el_type) = ElementType::try_from(possibly_el_type) {
                                    (Some(ident_val.to_string()), el_type, true)
                                } else {
                                    return Err(FoliumError::UnknownType {
                                        offending_token: possibly_el_type,
                                    });
                                }
                            }
                            Some(other_token) => {
                                return Err(FoliumError::ExpectedReason {
                                    expected: "a content type",
                                    got: other_token,
                                })
                            }
                        }
                    }
                    Some(other_token) => {
                        return Err(FoliumError::ExpectedToken {
                            expected: Definition,
                            got: other_token,
                        })
                    }
                    None => {
                        return Err(FoliumError::UnexpectedFileEndWithToken {
                            expected: Definition,
                        })
                    }
                }
            }
        }
        other_token => {
            return Err(FoliumError::ExpectedReason {
                expected: "a content type or name",
                got: other_token,
            })
        }
    };

    // Assert that this is followed by an OpeningArgsParen token ( if we
    // haven't done so already
    if should_check_opening_paren {
        match iter.next() {
            Some(OpeningArgsParen) => {}
            Some(other_token) => {
                return Err(FoliumError::ExpectedToken {
                    expected: OpeningArgsParen,
                    got: other_token,
                })
            }
            None => {
                return Err(FoliumError::UnexpectedFileEndWithToken {
                    expected: OpeningArgsParen,
                })
            }
        }
    }

    dbg!(&iter);

    let mut brackets: u8 = 1;
    let content_tokens = iter
        .take_while(|token| {
            // println!("token: {token:?}");
            match token {
                OpeningArgsParen => brackets += 1,
                ClosingArgsParen => brackets -= 1,
                _ => {}
            };
            brackets > 0
        })
        .collect::<Vec<_>>();

    dbg!(&content_tokens);

    Ok(match element_type {
        ElNone => global.push_element(AbstractElementData::None, element_type, maybe_name),
        Text => global.push_element(
            AbstractElementData::Text(match content_tokens[0] {
                Value(PropertyValue::String(ref s)) => s.clone(),
                _ => panic!("text content did not contain text value token"),
            }),
            element_type,
            maybe_name,
        ),
        Code => global.push_element(
            AbstractElementData::Code(match content_tokens[0] {
                Value(PropertyValue::String(ref s)) => s.clone(),
                _ => panic!("code content did not contain text value token"),
            }),
            element_type,
            maybe_name,
        ),
        Image => global.push_element(
            AbstractElementData::Image(match content_tokens[0] {
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
        Row => {
            let children_ids = content_tokens
                .split(|token| token == &ListSeparator)
                .map(|child_tokens| {
                    parse_content_definition(child_tokens.iter().cloned(), global)
                        .map_err(|err| {
                            eprintln!("{err}");
                            panic!();
                        })
                        .unwrap()
                })
                .collect::<Vec<_>>();
            global.push_element(
                AbstractElementData::Row(children_ids),
                element_type,
                maybe_name,
            )
        }
        Col => {
            let children_ids = content_tokens
                .split(|token| token == &ListSeparator)
                .map(|child_tokens| {
                    parse_content_definition(child_tokens.iter().cloned(), global)
                        .map_err(|err| {
                            eprintln!("{err}");
                            panic!();
                        })
                        .unwrap()
                })
                .collect::<Vec<_>>();
            global.push_element(
                AbstractElementData::Col(children_ids),
                element_type,
                maybe_name,
            )
        }
    })
}

pub fn load<P: AsRef<Path>>(global: &GlobalState, path: P) -> Result<(), FoliumError> {
    let source: &'static mut str = fs::read_to_string(path)
        .expect("could not open file")
        .lines()
        .filter(|l| !l.trim().starts_with("//"))
        .collect::<Vec<_>>()
        .join("")
        .replace('\n', "")
        .leak();
    let mut raw_tokens = source
        .split(' ')
        .flat_map(|s| s.split_inclusive(&['(', ')', '[', ']', '{', '}']))
        .flat_map(split_off_string_delims)
        .filter(|s| !s.is_empty())
        .map(|s| match s {
            "[" => RawToken::AlreadyParseable(OpeningSlideParen),
            "]" => RawToken::AlreadyParseable(ClosingSlideParen),
            "::" => RawToken::AlreadyParseable(Definition),
            ":" => RawToken::AlreadyParseable(ValueAssignment),
            "," => RawToken::AlreadyParseable(ListSeparator),
            "\"" => RawToken::AlreadyParseable(StringDelim),
            "(" => RawToken::AlreadyParseable(OpeningArgsParen),
            ")" => RawToken::AlreadyParseable(ClosingArgsParen),
            "{" => RawToken::AlreadyParseable(OpeningParamsParen),
            "}" => RawToken::AlreadyParseable(ClosingParamsParen),
            others => RawToken::NotYetParsed(others),
        });

    // unify strings and parse into proper tokens
    // TODO: don't turn multiple spaces into one

    let mut contiguous_tokens = Vec::new();
    let mut tokens_to_ignore: usize = 0;

    while let Some(next_raw_token) = raw_tokens.next() {
        // println!("current token: {:?}", next_raw_token);

        if tokens_to_ignore > 0 {
            // println!("should ignore {tokens_to_ignore} tokens, skipping");
            tokens_to_ignore -= 1;
            continue;
        }

        match next_raw_token {
            RawToken::AlreadyParseable(StringDelim) => {
                let string = raw_tokens
                    .clone()
                    .take_while(|elem| {
                        tokens_to_ignore += 1;
                        !matches!(elem, RawToken::AlreadyParseable(StringDelim))
                    })
                    .map(|elem| match elem {
                        RawToken::NotYetParsed(s) => s,
                        other => unreachable!(
                            "reached token {:?} while processing string. remaining tokens: {:?}",
                            other,
                            raw_tokens.clone().collect::<Vec<_>>()
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                contiguous_tokens.push(Value(PropertyValue::String(string)));
            }
            RawToken::AlreadyParseable(other) => {
                contiguous_tokens.push(other);
            }
            RawToken::NotYetParsed(val) => {
                if let Ok(number) = val.parse::<u32>() {
                    contiguous_tokens.push(Value(PropertyValue::Number(number)))
                } else if let Ok(boolean) = val.parse::<bool>() {
                    contiguous_tokens.push(Value(PropertyValue::Boolean(boolean)))
                } else {
                    contiguous_tokens.push(Ident(val))
                }
            }
        }
    }

    // group tokens by slide
    let mut grouped_tokens = Vec::new();
    let mut current_slide_tokens = Vec::new();

    for token in contiguous_tokens {
        match token {
            OpeningSlideParen => {}
            ClosingSlideParen => {
                grouped_tokens.push(current_slide_tokens.clone());
                current_slide_tokens.clear();
            }
            other => current_slide_tokens.push(other),
        }
    }

    for slide_tokens in grouped_tokens {
        // println!("{slide_tokens:?}");
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
                .split(|token| *token == ClosingParamsParen)
                .filter(|slice| !slice.is_empty());
            let mut style_map = StyleMap::new();

            for individual_style in individual_styles {
                let target = match &individual_style[0] {
                    &Ident(ident_val) => {
                        if let Ok(el_type) = ElementType::try_from(ident_val) {
                            StyleTarget::Anonymous(el_type)
                        } else {
                            StyleTarget::Named(ident_val.to_owned())
                        }
                    }
                    other_token => {
                        return Err(FoliumError::ExpectedReason {
                            expected: "a style target identifier",
                            got: other_token.clone(),
                        })
                    }
                };

                let properties: HashMap<String, PropertyValue> = individual_style[2..].chunks_exact(4).map(|slice| &slice[0..3]).map(|def| {
                    assert_eq!(def[1], Token::ValueAssignment);
                    (
                        match &def[0] {
                            Token::Ident(s) => s.to_string(),
                            other_token => panic!("found non-ident token {other_token:?} when parsing style directive")
                        },
                        match &def[2] {
                            Token::Value(pv) => pv,
                            other_token => panic!("found non-parameter value token {other_token:?} when parsing style directive")
                        }.clone(),
                    )
                }
                ).collect();

                match style_map.add_style(target, properties) {
                    Ok(_) => {}
                    Err(e) => panic!("{e}"),
                }
            }

            // make sure that properties like height and width are present if the user hasn't overridden them
            style_map.fill_in(StyleMap::default());

            style_map
        } else {
            StyleMap::default()
        };

        // dbg!(&style_map);

        let slide = Slide::new(global, content_root_id, style_map);
        global.push_slide(slide);
    }

    Ok(())
}
