use std::collections::HashMap;
use std::fmt::Display;

use crate::ast::{AbstractElementData, PropertyValue, PADDING_DUMMY};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StyleTarget {
    Named(String),
    Anonymous(AbstractElementData),
    Slide,
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

    pub fn add_style(
        &mut self,
        target: StyleTarget,
        properties: HashMap<String, PropertyValue>,
    ) -> Result<(), String> {
        if self.styles.contains_key(&target) {
            Err(String::from(
                "style target was already present in the style map",
            ))
        } else {
            self.styles.insert(target, properties);
            Ok(())
        }
    }

    pub fn fill_in(&mut self, other: Self) {
        for (target, properties) in other.styles {
            let _ = self.add_style(target, properties);
        }
    }

    pub fn styles_for_target<'a>(
        &'a self,
        target: StyleTarget,
    ) -> Option<&'a HashMap<String, PropertyValue>> {
        self.styles.get(&target)
    }
}

impl Default for StyleMap {
    fn default() -> Self {
        Self {
            styles: HashMap::from([
                (
                    StyleTarget::Slide,
                    HashMap::from([
                        (String::from("width"), PropertyValue::Number(1920)),
                        (String::from("height"), PropertyValue::Number(1080)),
                        (String::from("margin"), PropertyValue::Number(20)),
                    ]),
                ),
                (
                    StyleTarget::Anonymous(PADDING_DUMMY),
                    HashMap::from([(String::from("amount"), PropertyValue::Number(12))]),
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
        .expect(&format!("Property {property} was not found in style."))
    {
        PropertyValue::Number(val) => *val,
        PropertyValue::String(_) => panic!("Property {property} was found, but is of type String"),
        PropertyValue::Boolean(_) => {
            panic!("Property {property} was found, but is of type Boolean")
        }
    }
}

pub fn extract_string<S: Into<String> + Display>(
    map: &HashMap<String, PropertyValue>,
    property: S,
) -> String {
    match map
        .get(&property.to_string())
        .expect(&format!("Property {property} was not found in style."))
    {
        PropertyValue::Number(_) => panic!("Property {property} was found, but is of type Number"),
        PropertyValue::String(val) => val.to_owned(),
        PropertyValue::Boolean(_) => {
            panic!("Property {property} was found, but is of type Boolean")
        }
    }
}

pub fn extract_boolean<S: Into<String> + Display>(
    map: &HashMap<String, PropertyValue>,
    property: S,
) -> bool {
    match map
        .get(&property.to_string())
        .expect(&format!("Property {property} was not found in style."))
    {
        PropertyValue::Number(_) => panic!("Property {property} was found, but is of type Number"),
        PropertyValue::String(_) => panic!("Property {property} was found, but is of type String"),
        PropertyValue::Boolean(val) => *val,
    }
}
