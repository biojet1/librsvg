//! Representation of CSS types, and the CSS parsing and matching engine.
//!
//! # Terminology
//!
//! Consider a CSS **stylesheet** like this:
//!
//! ```ignore
//! @import url("another.css");
//!
//! foo, .bar {
//!         fill: red;
//!         stroke: green;
//! }
//!
//! #baz { stroke-width: 42; }
//!
//! ```
//! The example contains three **rules**, the first one is an **at-rule*,
//! the other two are **qualified rules**.
//!
//! Each rule is made of two parts, a **prelude** and an optional **block**
//! The prelude is the part until the first `{` or until `;`, depending on
//! whether a block is present.  The block is the part between curly braces.
//!
//! Let's look at each rule:
//!
//! `@import` is an **at-rule**.  This rule has a prelude, but no block.
//! There are other at-rules like `@media` and some of them may have a block,
//! but librsvg doesn't support those yet.
//!
//! The prelude of the following rule is `foo, .bar`.
//! It is a **selector list** with two **selectors**, one for
//! `foo` elements and one for elements that have the `bar` class.
//!
//! The content of the block between `{}` for a qualified rule is a
//! **declaration list**.  The block of the first qualified rule contains two
//! **declarations**, one for the `fill` **property** and one for the
//! `stroke` property.
//!
//! After ther first qualified rule, we have a second qualified rule with
//! a single selector for the `#baz` id, with a single declaration for the
//! `stroke-width` property.
//!
//! # Helper crates we use
//!
//! * `cssparser` crate as a CSS tokenizer, and some utilities to
//! parse CSS rules and declarations.
//!
//! * `selectors` crate for the representation of selectors and
//! selector lists, and for the matching engine.
//!
//! Both crates provide very generic implementations of their concepts,
//! and expect the caller to provide implementations of various traits,
//! and to provide types that represent certain things.
//!
//! For example, `cssparser` expects one to provide representations of
//! the following types:
//!
//! * A parsed CSS rule.  For `fill: blue;` we have
//! `ParsedProperty::Fill(...)`.
//!
//! * A declaration list; we use `DeclarationList`.
//!
//! * A parsed selector list; we use `SelectorList` from the
//! `selectors` crate.
//!
//! In turn, the `selectors` crate needs a way to navigate and examine
//! one's implementation of an element tree.  We provide `impl
//! selectors::Element for RsvgElement` for this.  This implementation
//! has methods like "does this element have the id `#foo`", or "give
//! me the next sibling element".
//!
//! Finally, the matching engine ties all of this together with
//! `matches_selector_list()`.  This takes an opaque representation of
//! an element, plus a selector list, and returns a bool.  We iterate
//! through the rules in a stylesheet and apply each rule that matches
//! to each element node.

use cssparser::*;
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::matching::{ElementSelectorFlags, MatchingContext, MatchingMode, QuirksMode};
use selectors::{self, OpaqueElement, SelectorImpl, SelectorList};

use std::collections::hash_map::Iter as HashMapIter;
use std::collections::HashMap;
use std::fmt;
use std::str;

use markup5ever::{namespace_url, ns, LocalName, Namespace, Prefix, QualName};
use url::Url;

use crate::allowed_url::AllowedUrl;
use crate::error::*;
use crate::io::{self, BinaryData};
use crate::node::{NodeType, RsvgNode};
use crate::properties::{parse_property, ParsedProperty};
use crate::text::NodeChars;

/// A parsed CSS declaration
///
/// For example, in the declaration `fill: green !important`, the
/// `prop_name` would be `fill`, the `property` would be
/// `ParsedProperty::Fill(...)` with the green value, and `important`
/// would be `true`.
pub struct Declaration {
    pub prop_name: QualName,
    pub property: ParsedProperty,
    pub important: bool,
}

/// A list of property/value declarations, hashed by the property name
///
/// For example, in a CSS rule:
///
/// ```ignore
/// foo { fill: red; stroke: green; }
/// ```
///
/// The stuff between braces is the declaration list; this example has two
/// declarations, one for `fill`, and one for `stroke`, each with its own value.
#[derive(Default)]
pub struct DeclarationList {
    // Maps property_name -> Declaration
    declarations: HashMap<QualName, Declaration>,
}

/// Dummy struct required to use `cssparser::DeclarationListParser`
///
/// It implements `cssparser::DeclarationParser`, which knows how to parse
/// the property/value pairs from a CSS declaration.
pub struct DeclParser;

impl<'i> DeclarationParser<'i> for DeclParser {
    type Declaration = Declaration;
    type Error = ValueErrorKind;

    /// Parses a CSS declaration like `name: input_value [!important]`
    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Declaration, cssparser::ParseError<'i, ValueErrorKind>> {
        let prop_name = QualName::new(None, ns!(svg), LocalName::from(name.as_ref()));
        let property =
            parse_property(&prop_name, input, true).map_err(|e| input.new_custom_error(e))?;

        let important = input.try_parse(parse_important).is_ok();

        Ok(Declaration {
            prop_name,
            property,
            important,
        })
    }
}

// cssparser's DeclarationListParser requires this; we just use the dummy
// implementations from cssparser itself.  We may want to provide a real
// implementation in the future, although this may require keeping track of the
// CSS parsing state like Servo does.
impl<'i> AtRuleParser<'i> for DeclParser {
    type PreludeBlock = ();
    type PreludeNoBlock = ();
    type AtRule = Declaration;
    type Error = ValueErrorKind;
}

/// Dummy struct to implement cssparser::QualifiedRuleParser
pub struct RuleParser;

/// Errors from the CSS parsing process
pub enum CssParseErrorKind<'i> {
    Selector(selectors::parser::SelectorParseErrorKind<'i>),
    Value(ValueErrorKind),
}

impl<'i> From<selectors::parser::SelectorParseErrorKind<'i>> for CssParseErrorKind<'i> {
    fn from(e: selectors::parser::SelectorParseErrorKind) -> CssParseErrorKind {
        CssParseErrorKind::Selector(e)
    }
}

/// A CSS qualified rule (or ruleset)
pub struct QualifiedRule {
    selectors: SelectorList<RsvgSelectors>,
    declarations: DeclarationList,
}

/// Prelude of at-rule used in the AtRuleParser.
pub enum AtRulePrelude {
    Import(String),
}

/// A CSS at-rule (or ruleset)
pub enum AtRule {
    Import(String),
}

/// A CSS rule (or ruleset)
pub enum Rule {
    AtRule(AtRule),
    QualifiedRule(QualifiedRule),
}

// Required to implement the `Prelude` associated type in `cssparser::QualifiedRuleParser`
impl<'i> selectors::Parser<'i> for RuleParser {
    type Impl = RsvgSelectors;
    type Error = CssParseErrorKind<'i>;

    fn default_namespace(&self) -> Option<<Self::Impl as SelectorImpl>::NamespaceUrl> {
        Some(ns!(svg))
    }

    fn namespace_for_prefix(
        &self,
        _prefix: &<Self::Impl as SelectorImpl>::NamespacePrefix,
    ) -> Option<<Self::Impl as SelectorImpl>::NamespaceUrl> {
        // FIXME: Do we need to keep a lookup table extracted from libxml2's
        // XML namespaces?
        //
        // Or are CSS namespaces completely different, declared elsewhere?
        None
    }
}

// `cssparser::RuleListParser` is a struct which requires that we
// provide a type that implements `cssparser::QualifiedRuleParser`.
//
// In turn, `cssparser::QualifiedRuleParser` requires that we
// implement a way to parse the `Prelude` of a ruleset or rule.  For
// example, in this ruleset:
//
// ```ignore
// foo, .bar { fill: red; stroke: green; }
// ```
//
// The prelude is the selector list with the `foo` and `.bar` selectors.
//
// The `parse_prelude` method just uses `selectors::SelectorList`.  This
// is what requires the `impl selectors::Parser for RuleParser`.
//
// Next, the `parse_block` method takes an already-parsed prelude (a selector list),
// and tries to parse the block between braces - a `DeclarationList`.  It creates
// a `Rule` out of the selector list and the declaration list.
impl<'i> QualifiedRuleParser<'i> for RuleParser {
    type Prelude = SelectorList<RsvgSelectors>;
    type QualifiedRule = Rule;
    type Error = CssParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
        SelectorList::parse(self, input)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _location: SourceLocation,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, cssparser::ParseError<'i, Self::Error>> {
        let declarations: HashMap<_, _> = DeclarationListParser::new(input, DeclParser)
            .into_iter()
            .filter_map(Result::ok) // ignore invalid property name or value
            .map(|decl| (decl.prop_name.clone(), decl))
            .collect();

        Ok(Rule::QualifiedRule(QualifiedRule {
            selectors: prelude,
            declarations: DeclarationList { declarations },
        }))
    }
}

// Required by `cssparser::RuleListParser`.
//
// This only handles the `@import` at-rule.
impl<'i> AtRuleParser<'i> for RuleParser {
    type PreludeBlock = ();
    type PreludeNoBlock = AtRulePrelude;
    type AtRule = Rule;
    type Error = CssParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<AtRuleType<Self::PreludeNoBlock, Self::PreludeBlock>, ParseError<'i, Self::Error>>
    {
        match_ignore_ascii_case! { &name,
            "import" => {
                // FIXME: at the moment we ignore media queries
                let url = input.expect_url_or_string()?.as_ref().to_owned();
                Ok(AtRuleType::WithoutBlock(AtRulePrelude::Import(url)))
            },

            _ => Err(input.new_error(BasicParseErrorKind::AtRuleInvalid(name))),
        }
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::PreludeNoBlock,
        _location: SourceLocation,
    ) -> Self::AtRule {
        let AtRulePrelude::Import(url) = prelude;
        Rule::AtRule(AtRule::Import(url))
    }
}

/// Dummy type required by the SelectorImpl trait.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonTSPseudoClass;

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        Ok(())
    }
}

impl selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = RsvgSelectors;

    fn is_active_or_hover(&self) -> bool {
        false
    }

    fn is_user_action_state(&self) -> bool {
        false
    }
}

/// Dummy type required by the SelectorImpl trait
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PseudoElement;

impl ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        Ok(())
    }
}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = RsvgSelectors;
}

/// Holds all the types for the SelectorImpl trait
#[derive(Debug, Clone)]
pub struct RsvgSelectors;

impl SelectorImpl for RsvgSelectors {
    type ExtraMatchingData = ();
    type AttrValue = String;
    type Identifier = LocalName;
    type ClassName = LocalName;
    type PartName = LocalName;
    type LocalName = LocalName;
    type NamespaceUrl = Namespace;
    type NamespacePrefix = Prefix;
    type BorrowedNamespaceUrl = Namespace;
    type BorrowedLocalName = LocalName;
    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

/// Wraps an `RsvgNode` with a locally-defined type, so we can implement
/// a foreign trait on it.
///
/// RsvgNode is an alias for rctree::Node, so we can't implement
/// `selectors::Element` directly on it.  We implement it on the
/// `RsvgElement` wrapper instead.
#[derive(Clone)]
pub struct RsvgElement(RsvgNode);

impl From<RsvgNode> for RsvgElement {
    fn from(n: RsvgNode) -> RsvgElement {
        RsvgElement(n)
    }
}

impl fmt::Debug for RsvgElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.borrow())
    }
}

// The selectors crate uses this to examine our tree of elements.
impl selectors::Element for RsvgElement {
    type Impl = RsvgSelectors;

    /// Converts self into an opaque representation.
    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(&self.0.borrow())
    }

    fn parent_element(&self) -> Option<Self> {
        self.0.parent().map(|n| n.into())
    }

    /// Whether the parent node of this element is a shadow root.
    fn parent_node_is_shadow_root(&self) -> bool {
        // unsupported
        false
    }

    /// The host of the containing shadow root, if any.
    fn containing_shadow_host(&self) -> Option<Self> {
        // unsupported
        None
    }

    /// Whether we're matching on a pseudo-element.
    fn is_pseudo_element(&self) -> bool {
        // unsupported
        false
    }

    /// Skips non-element nodes
    fn prev_sibling_element(&self) -> Option<Self> {
        let mut sibling = self.0.previous_sibling();

        while let Some(ref sib) = sibling {
            if sib.borrow().get_type() != NodeType::Chars {
                return sibling.map(|n| n.into());
            }

            sibling = self.0.previous_sibling();
        }

        None
    }

    /// Skips non-element nodes
    fn next_sibling_element(&self) -> Option<Self> {
        let mut sibling = self.0.next_sibling();

        while let Some(ref sib) = sibling {
            if sib.borrow().get_type() != NodeType::Chars {
                return sibling.map(|n| n.into());
            }

            sibling = self.0.next_sibling();
        }

        None
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &LocalName) -> bool {
        self.0.borrow().element_name().local == *local_name
    }

    /// Empty string for no namespace
    fn has_namespace(&self, ns: &Namespace) -> bool {
        self.0.borrow().element_name().ns == *ns
    }

    /// Whether this element and the `other` element have the same local name and namespace.
    fn is_same_type(&self, other: &Self) -> bool {
        self.0.borrow().element_name() == other.0.borrow().element_name()
    }

    fn attr_matches(
        &self,
        _ns: &NamespaceConstraint<&Namespace>,
        _local_name: &LocalName,
        _operation: &AttrSelectorOperation<&String>,
    ) -> bool {
        // unsupported
        false
    }

    fn match_non_ts_pseudo_class<F>(
        &self,
        _pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        _context: &mut MatchingContext<Self::Impl>,
        _flags_setter: &mut F,
    ) -> bool
    where
        F: FnMut(&Self, ElementSelectorFlags),
    {
        // unsupported
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        // unsupported
        false
    }

    /// Whether this element is a `link`.
    fn is_link(&self) -> bool {
        // FIXME: is this correct for SVG <a>, not HTML <a>?
        self.0.borrow().get_type() == NodeType::Link
    }

    /// Returns whether the element is an HTML <slot> element.
    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_id(&self, id: &LocalName, case_sensitivity: CaseSensitivity) -> bool {
        self.0
            .borrow()
            .get_id()
            .map(|self_id| case_sensitivity.eq(self_id.as_bytes(), id.as_ref().as_bytes()))
            .unwrap_or(false)
    }

    fn has_class(&self, name: &LocalName, case_sensitivity: CaseSensitivity) -> bool {
        self.0
            .borrow()
            .get_class()
            .map(|classes| {
                classes
                    .split_whitespace()
                    .any(|class| case_sensitivity.eq(class.as_bytes(), name.as_bytes()))
            })
            .unwrap_or(false)
    }

    fn is_part(&self, _name: &LocalName) -> bool {
        // unsupported
        false
    }

    /// Returns whether this element matches `:empty`.
    ///
    /// That is, whether it does not contain any child element or any non-zero-length text node.
    /// See http://dev.w3.org/csswg/selectors-3/#empty-pseudo
    fn is_empty(&self) -> bool {
        !self.0.has_children()
            || self.0.children().all(|child| {
                child.borrow().get_type() == NodeType::Chars
                    && child.borrow().get_impl::<NodeChars>().is_empty()
            })
    }

    /// Returns whether this element matches `:root`,
    /// i.e. whether it is the root element of a document.
    ///
    /// Note: this can be false even if `.parent_element()` is `None`
    /// if the parent node is a `DocumentFragment`.
    fn is_root(&self) -> bool {
        self.0.parent().is_none()
    }
}

impl DeclarationList {
    pub fn iter(&self) -> DeclarationListIter {
        DeclarationListIter(self.declarations.iter())
    }
}

/// Iterator for a `DeclarationList`, created with `decl_list.iter()`
pub struct DeclarationListIter<'a>(HashMapIter<'a, QualName, Declaration>);

impl<'a> Iterator for DeclarationListIter<'a> {
    type Item = &'a Declaration;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_attribute, declaration)| declaration)
    }
}

/// A parsed CSS stylesheet
#[derive(Default)]
pub struct Stylesheet {
    qualified_rules: Vec<QualifiedRule>,
}

impl Stylesheet {
    pub fn from_data(buf: &str, base_url: Option<&Url>) -> Result<Self, LoadingError> {
        let mut stylesheet = Stylesheet::default();
        stylesheet.parse(buf, base_url)?;
        Ok(stylesheet)
    }

    pub fn from_href(href: &str, base_url: Option<&Url>) -> Result<Self, LoadingError> {
        let mut stylesheet = Stylesheet::default();
        stylesheet.load(href, base_url)?;
        Ok(stylesheet)
    }

    /// Parses a CSS stylesheet from a string
    ///
    /// The `base_url` is required for `@import` rules, so that librsvg
    /// can determine if the requested path is allowed.
    fn parse(&mut self, buf: &str, base_url: Option<&Url>) -> Result<(), LoadingError> {
        let mut input = ParserInput::new(buf);
        let mut parser = Parser::new(&mut input);

        RuleListParser::new_for_stylesheet(&mut parser, RuleParser)
            .into_iter()
            .filter_map(Result::ok) // ignore invalid rules
            .for_each(|rule| match rule {
                Rule::AtRule(AtRule::Import(url)) => {
                    // ignore invalid imports
                    let _ = self.load(&url, base_url);
                }
                Rule::QualifiedRule(qr) => self.qualified_rules.push(qr),
            });

        Ok(())
    }

    /// Parses a stylesheet referenced by an URL
    fn load(&mut self, href: &str, base_url: Option<&Url>) -> Result<(), LoadingError> {
        let aurl = AllowedUrl::from_href(href, base_url).map_err(|_| LoadingError::BadUrl)?;

        io::acquire_data(&aurl, None)
            .and_then(|data| {
                let BinaryData {
                    data: bytes,
                    content_type,
                } = data;

                if content_type.as_ref().map(String::as_ref) == Some("text/css") {
                    Ok(bytes)
                } else {
                    rsvg_log!("\"{}\" is not of type text/css; ignoring", aurl);
                    Err(LoadingError::BadCss)
                }
            })
            .and_then(|bytes| {
                String::from_utf8(bytes).map_err(|_| {
                    rsvg_log!(
                        "\"{}\" does not contain valid UTF-8 CSS data; ignoring",
                        aurl
                    );
                    LoadingError::BadCss
                })
            })
            .and_then(|utf8| self.parse(&utf8, base_url))
    }

    /// The main CSS matching function.
    ///
    /// Takes a `node` and modifies its `specified_values` with the
    /// CSS rules that match the node.
    pub fn apply_matches_to_node(&self, node: &mut RsvgNode) {
        let mut match_ctx = MatchingContext::new(
            MatchingMode::Normal,

            // FIXME: how the fuck does one set up a bloom filter here?
            None,

            // n_index_cache,
            None,

            QuirksMode::NoQuirks,
        );

        for rule in &self.qualified_rules {
            if selectors::matching::matches_selector_list(
                &rule.selectors,
                &RsvgElement(node.clone()),
                &mut match_ctx,
            ) {
                for decl in rule.declarations.iter() {
                    node.borrow_mut().apply_style_declaration(decl);
                }
            }
        }
    }
}
