use std::cell::RefCell;
use std::path::PathBuf;

use crate::error::FoliumError;
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
        el_type: ElementType,
        name: Option<String>,
    ) -> AbstractElementID {
        let id = self.generate_id();
        self.elements.borrow_mut().push(AbstractElement {
            data,
            name,
            id,
            el_type,
        });

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
            .unwrap_or_else(|| panic!("{id} is not present"));
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

    pub fn number_of_slides(&self) -> usize {
        self.slides.borrow().len()
    }

    pub fn number_of_elements(&self) -> usize {
        self.elements.borrow().len()
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ElementType {
    Row,
    Col,
    Centre,
    Padding,
    Text,
    Code,
    Image,
    ElNone, // preferred naming over just None, which causes confusion with Option::None
}

impl ElementType {
    pub const fn string_rep(&self) -> &'static str {
        match self {
            ElementType::Row => "row",
            ElementType::Col => "col",
            ElementType::Centre => "centre",
            ElementType::Padding => "padding",
            ElementType::Text => "text",
            ElementType::Code => "code",
            ElementType::Image => "image",
            ElementType::ElNone => "none",
        }
    }
}

impl std::fmt::Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string_rep())
    }
}

impl<'a> TryFrom<&'a str> for ElementType {
    type Error = FoliumError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "col" => Ok(ElementType::Col),
            "row" => Ok(ElementType::Row),
            "text" => Ok(ElementType::Text),
            "code" => Ok(ElementType::Code),
            "img" => Ok(ElementType::Image),
            "none" => Ok(ElementType::ElNone),
            "padding" => Ok(ElementType::Padding),
            "centre" => Ok(ElementType::Centre),
            other => Err(FoliumError::UnknownType {
                offending_token: other,
                location: Default::default(),
            }),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AbstractElementID(pub u32);
impl std::fmt::Display for AbstractElementID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<ID {}>", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct AbstractElement {
    data: AbstractElementData,
    el_type: ElementType,
    id: AbstractElementID,
    name: Option<String>,
}

impl AbstractElement {
    pub fn data(&self) -> &AbstractElementData {
        &self.data
    }

    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn el_type(&self) -> ElementType {
        self.el_type
    }

    pub fn id(&self) -> AbstractElementID {
        self.id
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

    pub fn content(&self) -> AbstractElementID {
        self.content
    }

    pub fn id(&self) -> AbstractElementID {
        self.id
    }
}
