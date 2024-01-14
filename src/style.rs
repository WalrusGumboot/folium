use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

use strum::IntoEnumIterator;

use crate::ast::{AbstractElement, ElementType};
use crate::layout::SizeSpec;
use crate::{SLIDE_HEIGHT, SLIDE_WIDTH};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyValue {
    Number(u32),
    // Size(u32),
    String(String),
    Boolean(bool),
    Colour(u8, u8, u8),
    SizeSpec(SizeSpec),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StyleTarget {
    Named(String),
    Anonymous(ElementType),
    Slide,
}

impl StyleTarget {
    pub fn default_style(&self) -> HashMap<String, PropertyValue> {
        match self {
            StyleTarget::Named(..) => HashMap::new(),
            StyleTarget::Anonymous(el_type) => match el_type {
                ElementType::Sized => HashMap::new(),
                ElementType::Padding => {
                    HashMap::from([(String::from("amount"), PropertyValue::Number(12))])
                }
                ElementType::Row => {
                    HashMap::from([(String::from("gap"), PropertyValue::Number(32))])
                }
                ElementType::Col => {
                    HashMap::from([(String::from("gap"), PropertyValue::Number(32))])
                }
                ElementType::Centre => HashMap::new(),
                ElementType::Text => HashMap::from([
                    (String::from("size"), PropertyValue::Number(32)),
                    (
                        String::from("font"),
                        PropertyValue::String(String::from("Liberation Serif")),
                    ),
                    (String::from("fill"), PropertyValue::Colour(0, 0, 0)),
                ]),
                ElementType::Code => HashMap::from([
                    (String::from("bg"), PropertyValue::Colour(30, 30, 30)),
                    (String::from("fill"), PropertyValue::Colour(255, 255, 255)),
                    (String::from("margin"), PropertyValue::Number(20)),
                    (String::from("size"), PropertyValue::Number(32)),
                    (
                        String::from("font"),
                        PropertyValue::String(String::from("Liberation Mono")),
                    ),
                    (
                        String::from("language"),
                        PropertyValue::String(String::from("rs")),
                    ),
                ]),
                ElementType::Image => HashMap::new(),
                ElementType::ElNone => HashMap::new(),
            },
            StyleTarget::Slide => HashMap::from([
                (String::from("width"), PropertyValue::Number(SLIDE_WIDTH)),
                (String::from("height"), PropertyValue::Number(SLIDE_HEIGHT)),
                (String::from("margin"), PropertyValue::Number(64)),
                (String::from("bg"), PropertyValue::Colour(235, 218, 199)),
            ]),
        }
    }

    pub fn reify(elem: &AbstractElement) -> Self {
        match &elem.name() {
            Some(name) => Self::Named(name.to_owned()),
            None => Self::Anonymous(elem.el_type()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StyleMap {
    styles: HashMap<StyleTarget, HashMap<String, PropertyValue>>,
}

impl StyleMap {
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
        }
    }

    pub fn add_style(&mut self, target: StyleTarget, properties: HashMap<String, PropertyValue>) {
        self.styles.insert(target, properties);
    }

    pub fn fill_in(&mut self, other: Self) {
        for (target, properties) in other.styles {
            let existing_styles = self
                .styles
                .entry(target.clone())
                .or_insert(target.default_style());
            for (prop_name, prop_value) in properties {
                existing_styles.entry(prop_name).or_insert(prop_value);
            }
        }
    }

    pub fn styles_for_target(
        &self,
        target: &StyleTarget,
    ) -> Option<&HashMap<String, PropertyValue>> {
        self.styles.get(target)
    }
}

impl Default for StyleMap {
    fn default() -> Self {
        let mut style_map = StyleMap::new();
        style_map.add_style(StyleTarget::Slide, StyleTarget::Slide.default_style());
        for el in ElementType::iter() {
            style_map.add_style(
                StyleTarget::Anonymous(el),
                StyleTarget::Anonymous(el).default_style(),
            );
        }

        Self {
            styles: style_map.styles,
        }
    }
}

pub fn extract_number<S: Into<String> + Display>(
    map: &HashMap<String, PropertyValue>,
    property: S,
) -> u32 {
    match map
        .get(&property.to_string())
        .unwrap_or_else(|| panic!("Property {property} was not found in style."))
    {
        PropertyValue::Number(val) => *val,
        PropertyValue::String(_) => panic!("Property {property} was found, but is of type String"),
        PropertyValue::Boolean(_) => {
            panic!("Property {property} was found, but is of type Boolean")
        }
        PropertyValue::Colour(..) => {
            panic!("Property {property} was found, but is of type Colour")
        }
        PropertyValue::SizeSpec(_) => {
            panic!("Property {property} was found, but is of type SizeSpec")
        }
    }
}

pub fn extract_string<S: Into<String> + Display>(
    map: &HashMap<String, PropertyValue>,
    property: S,
) -> String {
    match map
        .get(&property.to_string())
        .unwrap_or_else(|| panic!("Property {property} was not found in style."))
    {
        PropertyValue::Number(_) => panic!("Property {property} was found, but is of type Number"),
        PropertyValue::String(val) => val.to_owned(),
        PropertyValue::Boolean(_) => {
            panic!("Property {property} was found, but is of type Boolean")
        }
        PropertyValue::Colour(..) => {
            panic!("Property {property} was found, but is of type Colour")
        }
        PropertyValue::SizeSpec(_) => {
            panic!("Property {property} was found, but is of type SizeSpec")
        }
    }
}

pub fn extract_boolean<S: Into<String> + Display>(
    map: &HashMap<String, PropertyValue>,
    property: S,
) -> bool {
    match map
        .get(&property.to_string())
        .unwrap_or_else(|| panic!("Property {property} was not found in style."))
    {
        PropertyValue::Number(_) => panic!("Property {property} was found, but is of type Number"),
        PropertyValue::String(_) => panic!("Property {property} was found, but is of type String"),
        PropertyValue::Boolean(val) => *val,
        PropertyValue::Colour(..) => {
            panic!("Property {property} was found, but is of type Colour")
        }
        PropertyValue::SizeSpec(_) => {
            panic!("Property {property} was found, but is of type SizeSpec")
        }
    }
}

pub fn extract_colour<S: Into<String> + Display>(
    map: &HashMap<String, PropertyValue>,
    property: S,
) -> (u8, u8, u8) {
    match map
        .get(&property.to_string())
        .unwrap_or_else(|| panic!("Property {property} was not found in style."))
    {
        PropertyValue::Number(_) => panic!("Property {property} was found, but is of type Number"),
        PropertyValue::String(_) => panic!("Property {property} was found, but is of type String"),
        PropertyValue::Boolean(_) => {
            panic!("Property {property} was found, but is of type Boolean")
        }
        PropertyValue::Colour(r, g, b) => (*r, *g, *b),
        PropertyValue::SizeSpec(_) => {
            panic!("Property {property} was found, but is of type SizeSpec")
        }
    }
}

pub fn extract_size_spec<S: Into<String> + Display>(
    map: &HashMap<String, PropertyValue>,
    property: S,
) -> SizeSpec {
    match map
        .get(&property.to_string())
        .unwrap_or_else(|| panic!("Property {property} was not found in style."))
    {
        PropertyValue::Number(_) => panic!("Property {property} was found, but is of type Number"),
        PropertyValue::String(_) => panic!("Property {property} was found, but is of type String"),
        PropertyValue::Boolean(_) => {
            panic!("Property {property} was found, but is of type Boolean")
        }
        PropertyValue::Colour(..) => {
            panic!("Property {property} was found, but is of type Colour")
        }
        PropertyValue::SizeSpec(spec) => *spec,
    }
}
