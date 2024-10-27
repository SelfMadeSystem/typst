//! HTML output.

mod dom;
mod elems;

pub use self::dom::*;

use ecow::EcoString;

use crate::foundations::{category, elem, Category, Content, Module, Scope};

/// HTML output.
#[category]
pub static HTML: Category;

/// Create a module with all HTML definitions.
pub fn module() -> Module {
    let mut html = Scope::deduplicating();
    html.category(HTML);
    html.define_elem::<HtmlElem>();
    self::elems::define(&mut html);
    Module::new("html", html)
}

/// A HTML element that can contain Typst content.
#[elem(name = "elem")]
pub struct HtmlElem {
    /// The element's tag.
    #[required]
    pub tag: HtmlTag,

    /// The element's attributes.
    #[borrowed]
    pub attrs: HtmlAttrs,

    /// The contents of the HTML element.
    #[positional]
    #[borrowed]
    pub body: Option<Content>,
}

impl HtmlElem {
    /// Add an atribute to the element.
    pub fn with_attr(
        mut self,
        key: impl Into<EcoString>,
        value: impl Into<EcoString>,
    ) -> Self {
        self.attrs
            .get_or_insert_with(Default::default)
            .0
            .push((key.into(), value.into()));
        self
    }
}
