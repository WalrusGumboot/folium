use std::{cell::RefCell, collections::HashMap, path::PathBuf};

#[derive(Clone, Debug)]
pub struct GlobalState {
    unassigned_id: RefCell<AbstractElementID>,
    slides: RefCell<Vec<Slide>>,
    elements: RefCell<Vec<AbstractElement>>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            unassigned_id: RefCell::new(AbstractElementID(0)),
            slides: RefCell::new(Vec::new()),
            elements: RefCell::new(Vec::new()),
        }
    }

    pub fn push_slide(&self, slide: Slide) {
        let mut slides = self.slides.borrow_mut();
        slides.push(slide);
    }

    pub fn push_element(
        &self,
        data: AbstractElementData,
        name: Option<String>,
    ) -> AbstractElementID {
        let id = self.generate_id();
        let mut elements = self.elements.borrow_mut();
        elements.push(AbstractElement { data, name, id });

        id
    }

    /// Because the first value returned by this function is AbstractElementID(1),
    /// an AbstractElementID of 0 is ALWAYS invalid and is used for a dummy referent.
    fn generate_id(&self) -> AbstractElementID {
        let mut id = self.unassigned_id.borrow_mut();
        *id = AbstractElementID(id.0 + 1);
        *id
    }
}

impl std::fmt::Display for GlobalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Global presentation state: {} slides, {} elements",
            self.slides.borrow().len(),
            self.elements.borrow().len()
        )?;
        writeln!(f, "Elements:")?;
        for elem in self.elements.borrow().iter() {
            writeln!(f, "    {elem:?}")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum AbstractElementData {
    Row(Vec<AbstractElementID>),
    Col(Vec<AbstractElementID>),
    Centre(AbstractElementID),
    Padding(AbstractElementID),
    Text(String),
    Code(String),
    Image(PathBuf),
    None,
}

pub const ROW_DUMMY: AbstractElementData = AbstractElementData::Row(Vec::new());
pub const COL_DUMMY: AbstractElementData = AbstractElementData::Col(Vec::new());
pub const CENTRE_DUMMY: AbstractElementData = AbstractElementData::Centre(AbstractElementID(0));
pub const PADDING_DUMMY: AbstractElementData = AbstractElementData::Padding(AbstractElementID(0));
pub const TEXT_DUMMY: AbstractElementData = AbstractElementData::Text(String::new());
pub const CODE_DUMMY: AbstractElementData = AbstractElementData::Code(String::new());
/// semantically equivalent to every instance of the None value but whatever
pub const NONE_DUMMY: AbstractElementData = AbstractElementData::None;
// pub const IMAGE_DUMMY: AbstractElementData = AbstractElementData::Image(PathBuf::from("/"));


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AbstractElementID(u32);

#[derive(Clone, Debug)]
pub struct AbstractElement {
    data: AbstractElementData,
    id: AbstractElementID,
    name: Option<String>,
}

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
            styles: HashMap::new()
        }
    }

    pub fn add_style(&mut self, target: StyleTarget, properties: HashMap<String, PropertyValue>) -> Result<(), String> {
        if self.styles.contains_key(&target) {
            Err(String::from("style target was already present in the style map"))
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
}

impl Default for StyleMap {
    fn default() -> Self {
        Self {
            styles: HashMap::from([(
                StyleTarget::Slide,
                HashMap::from([
                    (String::from("width"), PropertyValue::Number(1920)),
                    (String::from("height"), PropertyValue::Number(1080)),
                ]),
            )]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Slide {
    id: AbstractElementID,
    content: AbstractElementID,
    styles: StyleMap,
}

impl Slide {
    pub fn new(global: &GlobalState, content: AbstractElementID, styles: StyleMap) -> Self {
        Self {
            content,
            styles,
            id: global.generate_id(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Number(u32),
    // Size(u32),
    String(String),
    Boolean(bool),
}
