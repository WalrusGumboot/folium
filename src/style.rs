use std::collections::HashMap;
use std::fmt::Display;

use crate::ast::ElementType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyValue {
    Number(u32),
    // Size(u32),
    String(String),
    Boolean(bool),
    Colour(u8, u8, u8),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
                ElementType::Padding => {
                    HashMap::from([(String::from("amount"), PropertyValue::Number(12))])
                }
                ElementType::Row => {
                    HashMap::from([(String::from("gap"), PropertyValue::Number(6))])
                }
                ElementType::Col => {
                    HashMap::from([(String::from("gap"), PropertyValue::Number(6))])
                }
                ElementType::Centre => HashMap::new(),
                ElementType::Text => HashMap::from([
                    (String::from("size"), PropertyValue::Number(16)),
                    (
                        String::from("font"),
                        PropertyValue::String(String::from("Liberation Serif")),
                    ),
                    (String::from("fill"), PropertyValue::Colour(0, 0, 0))
                ]),
                ElementType::Code => HashMap::from([
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
                (String::from("width"), PropertyValue::Number(1920)),
                (String::from("height"), PropertyValue::Number(1080)),
                (String::from("margin"), PropertyValue::Number(20)),
                (String::from("bg"), PropertyValue::Colour(235, 218, 199)),
            ]),
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
        target: StyleTarget,
    ) -> Option<&HashMap<String, PropertyValue>> {
        self.styles.get(&target)
    }
}

impl Default for StyleMap {
    fn default() -> Self {
        // TODO: make this nicer, e.g. compile-time errors if new enum variants are ever introduced
        Self {
            styles: HashMap::from([
                (StyleTarget::Slide, StyleTarget::Slide.default_style()),
                (
                    StyleTarget::Anonymous(ElementType::Centre),
                    StyleTarget::Anonymous(ElementType::Centre).default_style(),
                ),
                (
                    StyleTarget::Anonymous(ElementType::Code),
                    StyleTarget::Anonymous(ElementType::Code).default_style(),
                ),
                (
                    StyleTarget::Anonymous(ElementType::Col),
                    StyleTarget::Anonymous(ElementType::Col).default_style(),
                ),
                (
                    StyleTarget::Anonymous(ElementType::ElNone),
                    StyleTarget::Anonymous(ElementType::ElNone).default_style(),
                ),
                (
                    StyleTarget::Anonymous(ElementType::Image),
                    StyleTarget::Anonymous(ElementType::Image).default_style(),
                ),
                (
                    StyleTarget::Anonymous(ElementType::Padding),
                    StyleTarget::Anonymous(ElementType::Padding).default_style(),
                ),
                (
                    StyleTarget::Anonymous(ElementType::Row),
                    StyleTarget::Anonymous(ElementType::Row).default_style(),
                ),
                (
                    StyleTarget::Anonymous(ElementType::Text),
                    StyleTarget::Anonymous(ElementType::Text).default_style(),
                ),
            ]),
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
    }
}
