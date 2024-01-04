use std::cell::RefCell;
use std::path::PathBuf;

use crate::style::StyleMap;

#[derive(Clone, Debug)]
pub struct GlobalState {
    unassigned_id: RefCell<AbstractElementID>,
    pub slides: RefCell<Vec<Slide>>,
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
        self.elements
            .borrow_mut()
            .push(AbstractElement { data, name, id });

        id
    }

    /// Because the first value returned by this function is AbstractElementID(1),
    /// an AbstractElementID of 0 is ALWAYS invalid and is used for a dummy referent.
    fn generate_id(&self) -> AbstractElementID {
        let mut id = self.unassigned_id.borrow_mut();
        *id = AbstractElementID(id.0 + 1);
        *id
    }

    pub fn get_element_by_id(&self, id: AbstractElementID) -> Option<AbstractElement> {
        self.elements
            .borrow()
            .iter()
            .find(|elem| elem.id == id)
            .cloned()
    }

    pub fn traverse(&self, id: AbstractElementID) -> Vec<AbstractElementID> {
        let elem = self
            .get_element_by_id(id)
            .expect(&format!("{id} is not present"));
        let all_children = match elem.data {
            AbstractElementData::Row(children) | AbstractElementData::Col(children) => children
                .into_iter()
                .flat_map(|child| self.traverse(child))
                .collect(),
            AbstractElementData::Centre(child) | AbstractElementData::Padding(child) => {
                self.traverse(child)
            }
            AbstractElementData::Text(_)
            | AbstractElementData::Code(_)
            | AbstractElementData::Image(_)
            | AbstractElementData::None => Vec::new(),
        };

        [[id].as_slice(), all_children.as_slice()].concat()
    }

    pub fn get_slide_elements(&self, slide: &Slide) -> Vec<AbstractElement> {
        let slide_root_id = slide.content;
        self.traverse(slide_root_id)
            .iter()
            .filter_map(|id| self.get_element_by_id(*id))
            .collect()
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
        writeln!(f, "Slides:")?;
        for elem in self.slides.borrow().iter() {
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
impl std::fmt::Display for AbstractElementID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<ID {}>", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct AbstractElement {
    data: AbstractElementData,
    id: AbstractElementID,
    name: Option<String>,
}

impl AbstractElement {
    pub fn data(&self) -> &AbstractElementData {
        &self.data
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

    pub fn style_map(&self) -> &StyleMap {
        &self.styles
    }

    pub fn id(&self) -> AbstractElementID {
        self.id
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Number(u32),
    // Size(u32),
    String(String),
    Boolean(bool),
}
