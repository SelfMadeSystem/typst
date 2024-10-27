//! Typst's HTML exporter.

mod encode;

pub use self::encode::html;

use comemo::{Track, Tracked, TrackedMut};
use typst_library::diag::{bail, warning, At, SourceResult};
use typst_library::engine::{Engine, Route, Sink, Traced};
use typst_library::foundations::{Content, StyleChain};
use typst_library::html::{HtmlDocument, HtmlElem, HtmlElement, HtmlNode};
use typst_library::introspection::{
    Introspector, Locator, LocatorLink, SplitLocator, TagElem,
};
use typst_library::model::{DocumentInfo, ParElem};
use typst_library::routines::{Arenas, Pair, RealizationKind, Routines};
use typst_library::text::{LinebreakElem, SmartQuoteElem, SpaceElem, TextElem};
use typst_library::World;
use typst_syntax::Span;

/// Produce an HTML document from content.
///
/// This first performs root-level realization and then turns the resulting
/// elements into HTML.
#[typst_macros::time(name = "html document")]
pub fn html_document(
    engine: &mut Engine,
    content: &Content,
    styles: StyleChain,
) -> SourceResult<HtmlDocument> {
    html_document_impl(
        engine.routines,
        engine.world,
        engine.introspector,
        engine.traced,
        TrackedMut::reborrow_mut(&mut engine.sink),
        engine.route.track(),
        content,
        styles,
    )
}

/// The internal implementation of `html_document`.
#[comemo::memoize]
#[allow(clippy::too_many_arguments)]
fn html_document_impl(
    routines: &Routines,
    world: Tracked<dyn World + '_>,
    introspector: Tracked<Introspector>,
    traced: Tracked<Traced>,
    sink: TrackedMut<Sink>,
    route: Tracked<Route>,
    content: &Content,
    styles: StyleChain,
) -> SourceResult<HtmlDocument> {
    let mut locator = Locator::root().split();
    let mut engine = Engine {
        routines,
        world,
        introspector,
        traced,
        sink,
        route: Route::extend(route).unnested(),
    };

    // Mark the external styles as "outside" so that they are valid at the page
    // level.
    let styles = styles.to_map().outside();
    let styles = StyleChain::new(&styles);

    let arenas = Arenas::default();
    let mut info = DocumentInfo::default();
    let children = (engine.routines.realize)(
        RealizationKind::HtmlDocument(&mut info),
        &mut engine,
        &mut locator,
        &arenas,
        content,
        styles,
    )?;

    // Convert the realized content into HTML nodes.
    let output = handle_list(&mut engine, &mut locator, children.iter().copied())?;

    // Ensure that the resulting HTML nodes are rooted.
    let root = root_element(output, &info)?;

    // TODO: Populate it.
    let introspector = Introspector::new(&[]);

    Ok(HtmlDocument { info, root, introspector })
}

/// Produce HTML nodes from content.
#[typst_macros::time(name = "html fragment")]
pub fn html_fragment(
    engine: &mut Engine,
    content: &Content,
    locator: Locator,
    styles: StyleChain,
) -> SourceResult<Vec<HtmlNode>> {
    html_fragment_impl(
        engine.routines,
        engine.world,
        engine.introspector,
        engine.traced,
        TrackedMut::reborrow_mut(&mut engine.sink),
        engine.route.track(),
        content,
        locator.track(),
        styles,
    )
}

/// The cached, internal implementation of [`html_fragment`].
#[comemo::memoize]
#[allow(clippy::too_many_arguments)]
fn html_fragment_impl(
    routines: &Routines,
    world: Tracked<dyn World + '_>,
    introspector: Tracked<Introspector>,
    traced: Tracked<Traced>,
    sink: TrackedMut<Sink>,
    route: Tracked<Route>,
    content: &Content,
    locator: Tracked<Locator>,
    styles: StyleChain,
) -> SourceResult<Vec<HtmlNode>> {
    let link = LocatorLink::new(locator);
    let mut locator = Locator::link(&link).split();
    let mut engine = Engine {
        routines,
        world,
        introspector,
        traced,
        sink,
        route: Route::extend(route),
    };

    engine.route.check_html_depth().at(content.span())?;

    let arenas = Arenas::default();
    let children = (engine.routines.realize)(
        RealizationKind::HtmlFragment,
        &mut engine,
        &mut locator,
        &arenas,
        content,
        styles,
    )?;

    handle_list(&mut engine, &mut locator, children.iter().copied())
}

/// Convert children into HTML nodes.
fn handle_list<'a>(
    engine: &mut Engine,
    locator: &mut SplitLocator,
    children: impl IntoIterator<Item = Pair<'a>>,
) -> SourceResult<Vec<HtmlNode>> {
    let mut output = Vec::new();
    for (child, styles) in children {
        handle(engine, child, locator, styles, &mut output)?;
    }
    Ok(output)
}

/// Convert a child into HTML node(s).
fn handle(
    engine: &mut Engine,
    child: &Content,
    locator: &mut SplitLocator,
    styles: StyleChain,
    output: &mut Vec<HtmlNode>,
) -> SourceResult<()> {
    if child.is::<TagElem>() {
        // TODO
    } else if let Some(elem) = child.to_packed::<HtmlElem>() {
        let mut children = vec![];
        if let Some(body) = elem.body(styles) {
            children = html_fragment(engine, body, locator.next(&elem.span()), styles)?;
        }
        let element = HtmlElement {
            tag: elem.tag.clone(),
            attrs: elem.attrs(styles).clone(),
            children,
            span: elem.span(),
        };
        output.push(element.into());
    } else if let Some(elem) = child.to_packed::<ParElem>() {
        let children = handle_list(engine, locator, elem.children.iter(&styles))?;
        output.push(HtmlElement::new("p", elem.span()).with_children(children).into());
    } else if child.is::<SpaceElem>() {
        output.push(HtmlNode::text(' '));
    } else if let Some(elem) = child.to_packed::<TextElem>() {
        output.push(HtmlNode::text(elem.text.clone()));
    } else if let Some(elem) = child.to_packed::<LinebreakElem>() {
        output.push(HtmlElement::new("br", elem.span()).into());
    } else if let Some(elem) = child.to_packed::<SmartQuoteElem>() {
        output.push(HtmlNode::text(if elem.double(styles) { '"' } else { '\'' }));
    } else {
        engine.sink.warn(warning!(
            child.span(),
            "{} was ignored during HTML export",
            child.elem().name()
        ));
    }
    Ok(())
}

/// Wrap the nodes in `<html>` and `<body>` if they are not yet rooted,
/// supplying a suitable `<head>`.
fn root_element(output: Vec<HtmlNode>, info: &DocumentInfo) -> SourceResult<HtmlElement> {
    let body = match classify_output(output)? {
        OutputKind::Html(element) => return Ok(element),
        OutputKind::Body(body) => body,
        OutputKind::Leafs(leafs) => {
            HtmlElement::new("body", Span::detached()).with_children(leafs)
        }
    };
    Ok(HtmlElement::new("html", Span::detached())
        .with_children(vec![head_element(info).into(), body.into()]))
}

/// Generate a `<head>` element.
fn head_element(info: &DocumentInfo) -> HtmlElement {
    let mut children = vec![];
    if let Some(title) = &info.title {
        children.push(
            HtmlElement::new("title", Span::detached())
                .with_children(vec![HtmlNode::Text(title.clone())])
                .into(),
        );
    }
    HtmlElement::new("head", Span::detached()).with_children(children)
}

/// Determine which kind of output the user generated.
fn classify_output(mut output: Vec<HtmlNode>) -> SourceResult<OutputKind> {
    let len = output.len();
    for node in &mut output {
        let HtmlNode::Element(elem) = node else { continue };
        let tag = elem.tag.as_str();
        let mut take = || std::mem::replace(elem, HtmlElement::new("", Span::detached()));
        match (tag, len) {
            ("html", 1) => return Ok(OutputKind::Html(take())),
            ("body", 1) => return Ok(OutputKind::Body(take())),
            ("html" | "body", _) => bail!(
                elem.span,
                "`{}` element must be the only element in the document",
                elem.tag
            ),
            _ => {}
        }
    }
    Ok(OutputKind::Leafs(output))
}

/// What kinds of output the user generated.
enum OutputKind {
    /// The user generated their own `<html>` element. We do not need to supply
    /// one.
    Html(HtmlElement),
    /// The user generate their own `<body>` element. We do not need to supply
    /// one, but need supply the `<html>` element.
    Body(HtmlElement),
    /// The user generated leafs which we wrap in a `<body>` and `<html>`.
    Leafs(Vec<HtmlNode>),
}
