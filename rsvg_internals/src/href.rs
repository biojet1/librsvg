//! Handling of `xlink:href` and `href` attributes
//!
//! In SVG1.1, links to elements are done with the `xlink:href` attribute.  However, SVG2
//! reduced this to just plain `href` with no namespace:
//! https://svgwg.org/svg2-draft/linking.html#XLinkRefAttrs
//!
//! If an element has both `xlink:href` and `href` attributes, the `href` overrides the
//! other.  We implement that logic in this module.

use markup5ever::{ExpandedName, expanded_name, local_name, namespace_url, ns};

/// Returns whether the attribute is either of `xlink:href` or `href`.
///
/// # Example
///
/// Use with an `if` pattern inside a `match`:
///
/// ```ignore
/// # use markup5ever::{LocalName, Namespace, Prefix, QualName, namespace_url};
/// let qual_name = QualName::new(
///     Some(Prefix::from("xlink")),
///     Namespace::from("http://www.w3.org/1999/xlink"),
///     LocalName::from("href"),
/// );
///
/// // assume foo is an Option<Value>
/// // assume value is a Value
///
/// match qual_name.expanded() {
///     ref name if is_href(name) => set_href(name, &mut foo, value),
///     _ => unreachable!(),
/// }
/// ```
pub fn is_href(name: &ExpandedName) -> bool {
    match *name {
        expanded_name!(xlink "href")
            | expanded_name!("", "href") => true,
        _ => false,
    }
}

/// Sets an `href` attribute in preference over an `xlink:href` one.
///
/// See [`is_href`](#fn.is_href.html) for example usage.
pub fn set_href<T>(name: &ExpandedName, dest: &mut Option<T>, href: T) {
    if dest.is_none() || *name != expanded_name!(xlink "href") {
        *dest = Some(href);
    }
}
