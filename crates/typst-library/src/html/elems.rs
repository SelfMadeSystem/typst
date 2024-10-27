use ecow::EcoString;
use typst_syntax::Span;

use crate::foundations::{func, Content, NativeElement, Scope};
use crate::html::HtmlElem;

macro_rules! elems {
    ($($tag:ident $(($($attr:ident),*))?)*) => {
        pub(super) fn define(html: &mut Scope) {
            $(html.define_func::<$tag>();)*
        }

        $(
            #[doc = "Produces an HTML `<"]
            #[doc = stringify!("$tag")]
            #[doc = ">` element."]
            #[func]
            fn $tag(
                span: Span,
                $($(
                    #[named]
                    $attr: Option<EcoString>,
                )*)?
                #[default]
                body: Option<Option<Content>>,
            ) -> Content {
                let mut elem = HtmlElem::new(stringify!($tag).into());
                $($(if let Some($attr) = $attr {
                    elem = elem.with_attr(stringify!($attr), $attr);
                })*)?
                if let Some(body) = body {
                    elem.push_body(body);
                }
                elem.pack().spanned(span)
            }
        )*
    }
}

// TODO: The attributes are totally incomplete. This is just for demonstration.
elems! {
    a
    abbr
    address
    area
    article
    aside
    audio
    b
    base
    bdi
    bdo
    blockquote
    body
    br
    button
    canvas
    caption
    cite
    code
    col
    colgroup
    data
    datalist
    dd
    del
    details
    dfn
    dialog
    div (style, width, height)
    dl
    dt
    em
    embed
    fieldset
    figcaption
    figure
    footer
    form
    h1
    head
    header
    hgroup
    hr
    html
    i
    iframe
    img (src, width, height)
    input
    ins
    kbd
    label
    legend
    li
    link
    main
    map
    mark
    menu
    meta
    meter
    nav
    noscript
    object
    ol
    optgroup
    option
    output
    p
    picture
    pre
    progress
    q
    rp
    rt
    ruby
    s
    samp
    script
    search
    section
    select
    slot
    small
    source
    span
    strong
    style
    sub
    summary
    sup
    table
    tbody
    td
    template
    textarea
    tfoot
    th
    thead
    time
    title
    tr
    track
    u
    ul
    var
    video
    wbr
}
