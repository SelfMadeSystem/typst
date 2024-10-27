use std::fmt::{self, Debug, Display, Formatter};

use ecow::{EcoString, EcoVec};
use typst_syntax::Span;
use typst_utils::PicoStr;

use crate::diag::HintedStrResult;
use crate::foundations::{cast, Dict};
use crate::introspection::Introspector;
use crate::model::DocumentInfo;

/// An HTML document.
#[derive(Debug, Clone)]
pub struct HtmlDocument {
    /// The document's root HTML element.
    pub root: HtmlElement,
    /// Details about the document.
    pub info: DocumentInfo,
    /// Provides the ability to execute queries on the document.
    pub introspector: Introspector,
}

/// A child of an HTML element.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum HtmlNode {
    /// Plain text.
    Text(EcoString),
    /// Another element.
    Element(HtmlElement),
}

impl HtmlNode {
    /// Create a plain text node.
    pub fn text(text: impl Into<EcoString>) -> Self {
        Self::Text(text.into())
    }
}

impl From<HtmlElement> for HtmlNode {
    fn from(element: HtmlElement) -> Self {
        Self::Element(element)
    }
}

/// An HTML element.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct HtmlElement {
    /// The HTML tag.
    pub tag: HtmlTag,
    /// The element's attributes.
    pub attrs: HtmlAttrs,
    /// The element's children.
    pub children: Vec<HtmlNode>,
    /// The span from which the element originated, if any.
    pub span: Span,
}

impl HtmlElement {
    /// Create a new, blank element without attributes or children.
    pub fn new(tag: impl Into<HtmlTag>, span: Span) -> Self {
        Self {
            tag: tag.into(),
            attrs: HtmlAttrs::default(),
            children: vec![],
            span,
        }
    }

    /// Attach children to the element.
    ///
    /// Note: This overwrites potential previous children.
    pub fn with_children(mut self, children: Vec<HtmlNode>) -> Self {
        self.children = children;
        self
    }
}

/// The tag of an HTML element.
#[derive(Clone, PartialEq, Hash)]
pub struct HtmlTag(pub PicoStr);

impl HtmlTag {
    /// Resolves the tag to a string.
    pub fn as_str(&self) -> &'static str {
        self.0.resolve()
    }

    /// Turns the tag into its inner interned string.
    pub fn into_inner(self) -> PicoStr {
        self.0
    }

    /// Whether the element is inline-level as opposed to being block-level.
    ///
    /// Not sure whether this distinction really makes sense. But we somehow
    /// need to decide what to put into automatic paragraphs. A `<strong>`
    /// should merged into a paragraph created by realization, but a `<div>`
    /// shouldn't.
    ///
    /// https://www.w3.org/TR/html401/struct/global.html#block-inline
    /// https://developer.mozilla.org/en-US/docs/Glossary/Inline-level_content
    /// https://github.com/orgs/mdn/discussions/353
    pub fn is_inline(&self) -> bool {
        matches!(
            self.as_str(),
            "abbr"
                | "a"
                | "bdi"
                | "b"
                | "br"
                | "bdo"
                | "code"
                | "cite"
                | "dfn"
                | "data"
                | "i"
                | "em"
                | "mark"
                | "kbd"
                | "rp"
                | "q"
                | "ruby"
                | "rt"
                | "samp"
                | "s"
                | "span"
                | "small"
                | "sub"
                | "strong"
                | "time"
                | "sup"
                | "var"
                | "u"
        )
    }
}

impl Debug for HtmlTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for HtmlTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.as_str())
    }
}

cast! {
    HtmlTag,
    self => self.0.into_value(),
    v: PicoStr => Self(v),
}

impl<S> From<S> for HtmlTag
where
    S: Into<PicoStr>,
{
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

/// Attributes of an HTML element.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct HtmlAttrs(pub EcoVec<(EcoString, EcoString)>);

cast! {
    HtmlAttrs,
    self => self.0
        .into_iter()
        .map(|(key, value)| (key.into(), value.into_value()))
        .collect::<Dict>()
        .into_value(),
    values: Dict => Self(values
        .into_iter()
        .map(|(k, v)| {
            let value = v.cast::<EcoString>()?;
            Ok((k.into(), value))
        })
        .collect::<HintedStrResult<_>>()?),
}
