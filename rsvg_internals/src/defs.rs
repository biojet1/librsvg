use libc;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;

use allowed_url::AllowedUrl;
use handle::{self, RsvgHandle};
use node::{Node, RsvgNode};
use util::utf8_cstr;

pub enum RsvgDefs {}

pub struct Defs {
    nodes: HashMap<String, Rc<Node>>,
    externs: HashMap<String, *const RsvgHandle>,
}

impl Defs {
    pub fn new() -> Defs {
        Defs {
            nodes: Default::default(),
            externs: Default::default(),
        }
    }

    pub fn insert(&mut self, id: &str, node: &Rc<Node>) {
        self.nodes.entry(id.to_string()).or_insert(node.clone());
    }

    /// Returns a node from an URI reference, or `None`
    ///
    /// This may return a node within the same RSVG handle, or a node in a secondary RSVG
    /// handle that is referenced by the current one.  If the element's id is not found,
    /// returns `None`.
    pub fn lookup(&mut self, handle: *const RsvgHandle, reference: &Href) -> Option<&Rc<Node>> {
        match reference {
            Href::PlainUri(_) => None,
            Href::FragmentId(ref fragment) => self.nodes.get(fragment),
            Href::UriWithFragmentId(ref href, ref fragment) => {
                match self.get_extern_handle(handle, href) {
                    Ok(extern_handle) => handle::get_defs(extern_handle).nodes.get(fragment),
                    Err(()) => None,
                }
            }
        }
    }

    fn get_extern_handle(
        &mut self,
        handle: *const RsvgHandle,
        href: &str,
    ) -> Result<*const RsvgHandle, ()> {
        let aurl =
            AllowedUrl::from_href(href, handle::get_base_url(handle).as_ref()).map_err(|_| ())?;

        match self.externs.entry(aurl.url().as_str().to_string()) {
            Entry::Occupied(e) => Ok(*(e.get())),
            Entry::Vacant(e) => {
                let extern_handle = handle::load_extern(handle, e.key())?;
                e.insert(extern_handle);
                Ok(extern_handle)
            }
        }
    }
}

/// Represents a possibly non-canonical URI with an optional fragment identifier
///
/// Sometimes in SVG element references (e.g. the `href` in the `<feImage>` element) we
/// must decide between referencing an external file, or using a plain fragment identifier
/// like `href="#foo"` as a reference to an SVG element in the same file as the one being
/// processes.  This enum makes that distinction.
#[derive(Debug, PartialEq)]
pub enum Href {
    PlainUri(String),
    FragmentId(String),
    UriWithFragmentId(String, String),
}

/// Errors returned when creating an `Href` out of a string
#[derive(Debug, PartialEq)]
pub enum HrefError {
    /// The href is an invalid URI or has empty components.
    ParseError,

    /// A fragment identifier ("`#foo`") is not allowed here
    ///
    /// For example, the SVG `<image>` element only allows referencing
    /// resources without fragment identifiers like
    /// `xlink:href="foo.png"`.
    FragmentForbidden,

    /// A fragment identifier ("`#foo`") was required but not found.  For example,
    /// the SVG `<use>` element requires one, as in `<use xlink:href="foo.svg#bar">`.
    FragmentRequired,
}

impl Href {
    /// Parses a string into an Href, or returns an error
    ///
    /// An href can come from an `xlink:href` attribute in an SVG
    /// element.  This function determines if the provided href is a
    /// plain absolute or relative URL ("`foo.png`"), or one with a
    /// fragment identifier ("`foo.svg#bar`").
    pub fn parse(href: &str) -> Result<Href, HrefError> {
        let (uri, fragment) = match href.rfind('#') {
            None => (Some(href), None),
            Some(p) if p == 0 => (None, Some(&href[1..])),
            Some(p) => (Some(&href[..p]), Some(&href[(p + 1)..])),
        };

        match (uri, fragment) {
            (None, Some(f)) if f.len() == 0 => Err(HrefError::ParseError),
            (None, Some(f)) => Ok(Href::FragmentId(f.to_string())),
            (Some(u), _) if u.len() == 0 => Err(HrefError::ParseError),
            (Some(u), None) => Ok(Href::PlainUri(u.to_string())),
            (Some(_u), Some(f)) if f.len() == 0 => Err(HrefError::ParseError),
            (Some(u), Some(f)) => Ok(Href::UriWithFragmentId(u.to_string(), f.to_string())),
            (_, _) => Err(HrefError::ParseError),
        }
    }

    pub fn without_fragment(href: &str) -> Result<Href, HrefError> {
        use self::Href::*;

        match Href::parse(href)? {
            r @ PlainUri(_) => Ok(r),
            FragmentId(_) | UriWithFragmentId(_, _) => Err(HrefError::FragmentForbidden),
        }
    }

    pub fn with_fragment(href: &str) -> Result<Href, HrefError> {
        use self::Href::*;

        match Href::parse(href)? {
            PlainUri(_) => Err(HrefError::FragmentRequired),
            r @ FragmentId(_) => Ok(r),
            r @ UriWithFragmentId(_, _) => Ok(r),
        }
    }
}

#[no_mangle]
pub extern "C" fn rsvg_defs_free(defs: *mut RsvgDefs) {
    assert!(!defs.is_null());

    unsafe {
        let defs = { &mut *(defs as *mut Defs) };
        Box::from_raw(defs);
    }
}

#[no_mangle]
pub extern "C" fn rsvg_defs_lookup(
    defs: *mut RsvgDefs,
    handle: *const RsvgHandle,
    name: *const libc::c_char,
) -> *const RsvgNode {
    assert!(!defs.is_null());
    assert!(!name.is_null());

    let defs = unsafe { &mut *(defs as *mut Defs) };
    let name = unsafe { utf8_cstr(name) };

    let r = Href::parse(name);
    if r.is_err() {
        return ptr::null();
    }

    match defs.lookup(handle, &r.unwrap()) {
        Some(n) => n as *const RsvgNode,
        None => ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            Href::parse("uri").unwrap(),
            Href::PlainUri("uri".to_string())
        );
        assert_eq!(
            Href::parse("#fragment").unwrap(),
            Href::FragmentId("fragment".to_string())
        );
        assert_eq!(
            Href::parse("uri#fragment").unwrap(),
            Href::UriWithFragmentId("uri".to_string(), "fragment".to_string())
        );
    }

    #[test]
    fn parse_errors() {
        assert_eq!(Href::parse(""), Err(HrefError::ParseError));
        assert_eq!(Href::parse("#"), Err(HrefError::ParseError));
        assert_eq!(Href::parse("uri#"), Err(HrefError::ParseError));
    }

    #[test]
    fn without_fragment() {
        assert_eq!(
            Href::without_fragment("uri").unwrap(),
            Href::PlainUri("uri".to_string())
        );

        assert_eq!(
            Href::without_fragment("#foo"),
            Err(HrefError::FragmentForbidden)
        );

        assert_eq!(
            Href::without_fragment("uri#foo"),
            Err(HrefError::FragmentForbidden)
        );
    }

    #[test]
    fn with_fragment() {
        assert_eq!(
            Href::with_fragment("#foo").unwrap(),
            Href::FragmentId("foo".to_string())
        );

        assert_eq!(
            Href::with_fragment("uri#foo").unwrap(),
            Href::UriWithFragmentId("uri".to_string(), "foo".to_string())
        );

        assert_eq!(Href::with_fragment("uri"), Err(HrefError::FragmentRequired));
    }
}
