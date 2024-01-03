use std::path::Path;
use std::{collections::HashMap, fs};

use crate::ast::{
    AbstractElementData, AbstractElementID, GlobalState, PropertyValue, Slide, StyleMap,
    StyleTarget, CENTRE_DUMMY, CODE_DUMMY, COL_DUMMY, NONE_DUMMY, PADDING_DUMMY, ROW_DUMMY,
    TEXT_DUMMY,
};

#[derive(Clone, Debug, PartialEq)]
enum Token<'a> {
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
    if s == "::" { return vec!["::"] } 
    let mut ret = Vec::new();
    if let Some(new_s) = s.strip_prefix('"') {
        s = new_s;
        ret.push("\"");
    }

    if let Some(new_s) = s.strip_suffix("\",") {
        ret.push(new_s);
        ret.push("\"");
        ret.push(",")
    } else {
        if let Some(new_s) = s.strip_suffix('"') {
            ret.push(new_s);
            ret.push("\"")
        } else {
            if let Some(new_s) = s.strip_suffix(',') {
                ret.push(new_s);
                ret.push(",")
            } else {
                if let Some(new_s) = s.strip_suffix(':') {
                    ret.push(new_s);
                    ret.push(":")
                } else {
                    ret.push(s);
                }
            }
        }
    }

    ret
}

/// Takes an iterator of tokens and returns the defined AbstractElement
fn parse_content_definition<'a, I: std::fmt::Debug + Iterator<Item = Token<'a>>>(
    mut iter: I,
    global: &'a GlobalState,
) -> AbstractElementID {
    let content_name_or_type = iter
        .next()
        .expect("could not parse name of following content item");
    // TODO: check for name duplicates
    let (maybe_name, content_type) = match content_name_or_type {
        Ident("col") => (None, "col"),
        Ident("row") => (None, "row"),
        Ident("text") => (None, "text"),
        Ident("code") => (None, "code"),
        Ident("img") => (None, "img"),
        Ident("none") => (None, "none"),
        Ident("padding") => (None, "padding"),
        Ident("centre") => (None, "centre"),
        Ident(other) => {
            // println!("skipped {:?}", iter.next());
            iter.next();
            // NOTE: if an anonymous element of an unknown type is given, this branch is hit too
            // it will fail due to the expect on the line below, but it's kind of ugly nonetheless
            let content_type = match iter
                .next()
                .expect("could not find next token to parse type")
            {
                Ident(val) => val,
                other => panic!("token expected to be of type but was {other:?} instead"),
            };
            (Some(String::from(other)), content_type)
        }
        other_token => panic!("invalid token found in content definition: {other_token:?}"),
    };

    // println!("element will have name {maybe_name:?} and type {content_type}");
    assert_eq!(iter.next().unwrap(), OpeningArgsParen);

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

    // println!("{content_tokens:?}");

    match content_type {
        "none" => global.push_element(AbstractElementData::None, maybe_name),
        "text" => global.push_element(
            AbstractElementData::Text(match content_tokens[0] {
                Value(PropertyValue::String(ref s)) => s.clone(),
                _ => panic!("text content did not contain text value token"),
            }),
            maybe_name,
        ),
        "code" => global.push_element(
            AbstractElementData::Code(match content_tokens[0] {
                Value(PropertyValue::String(ref s)) => s.clone(),
                _ => panic!("code content did not contain text value token"),
            }),
            maybe_name,
        ),
        "img" => global.push_element(
            AbstractElementData::Image(match content_tokens[0] {
                Value(PropertyValue::String(ref s)) => s.clone().into(),
                _ => panic!("img content did not contain text value token"),
            }),
            maybe_name,
        ),
        "centre" => global.push_element(
            AbstractElementData::Centre(parse_content_definition(
                content_tokens.into_iter(),
                global,
            )),
            maybe_name,
        ),
        "padding" => global.push_element(
            AbstractElementData::Padding(parse_content_definition(
                content_tokens.into_iter(),
                global,
            )),
            maybe_name,
        ),
        "row" => {
            let children_ids = content_tokens
                .split(|token| token == &ListSeparator)
                .map(|child_tokens| {
                    parse_content_definition(child_tokens.into_iter().cloned(), global)
                })
                .collect::<Vec<_>>();
            global.push_element(AbstractElementData::Row(children_ids), maybe_name)
        }
        "col" => {
            let children_ids = content_tokens
                .split(|token| token == &ListSeparator)
                .map(|child_tokens| {
                    parse_content_definition(child_tokens.into_iter().cloned(), global)
                })
                .collect::<Vec<_>>();
            global.push_element(AbstractElementData::Col(children_ids), maybe_name)
        }
        _ => unreachable!("hit content type for which no parser exists"),
    }
}

pub fn load<'a, P: AsRef<Path>>(global: &'a GlobalState, path: P) -> Result<(), String> {
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
        .filter(|s| *s != "")
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
        let content_root_id = parse_content_definition(&mut iter, &global);

        let remaining_style_tokens = iter.collect::<Vec<_>>();

        let style_map: StyleMap = if remaining_style_tokens.len() > 0 {
            let individual_styles = remaining_style_tokens
                .split(|token| *token == ClosingParamsParen)
                .filter(|slice| !slice.is_empty());
            let mut style_map = StyleMap::new();

            for individual_style in individual_styles {
                let target = match &individual_style[0] {
                    &Ident("slide") => StyleTarget::Slide,
                    &Ident("row") => StyleTarget::Anonymous(ROW_DUMMY),
                    &Ident("col") => StyleTarget::Anonymous(COL_DUMMY),
                    &Ident("centre") => StyleTarget::Anonymous(CENTRE_DUMMY),
                    &Ident("padding") => StyleTarget::Anonymous(PADDING_DUMMY),
                    &Ident("text") => StyleTarget::Anonymous(TEXT_DUMMY),
                    &Ident("code") => StyleTarget::Anonymous(CODE_DUMMY),
                    &Ident("none") => StyleTarget::Anonymous(NONE_DUMMY),
                    &Ident("img") => todo!(),
                    &Ident(name) => StyleTarget::Named(name.to_string()),
                    other_token => unreachable!(
                        "found non-ident token {other_token:?} while parsing style data"
                    ),
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

        dbg!(&style_map);

        let slide = Slide::new(&global, content_root_id, style_map);
        global.push_slide(slide);
    }

    Ok(())
}
