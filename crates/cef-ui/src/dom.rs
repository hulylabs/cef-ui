use std::{collections::HashMap, mem::zeroed};

use anyhow::Result;

use cef_ui_sys::{
    _cef_domdocument_t, _cef_domvisitor_t, cef_domdocument_t, cef_domnode_t, cef_domvisitor_t
};

use crate::{
    CefString, CefStringMap, DOMDocumentType, DOMFormControlType, DOMNodeType, Rect, RefCountedPtr,
    Wrappable, Wrapped, ref_counted_ptr, try_c
};

/// Interface to implement for visiting the DOM. The methods of this class will
/// be called on the render process main thread.
pub trait DOMVisitorCallbacks: Send + Sync + 'static {
    /// Method executed for visiting the DOM. The document object passed to this
    /// method represents a snapshot of the DOM at the time this method is
    /// executed. DOM objects are only valid for the scope of this method. Do not
    /// keep references to or attempt to access any DOM objects outside the scope
    /// of this method.
    fn visit(&mut self, document: DOMDocument);
}

ref_counted_ptr!(DOMVisitor, cef_domvisitor_t);

impl DOMVisitor {
    pub fn new<C: DOMVisitorCallbacks>(delegate: C) -> Self {
        Self(DOMVisitorWrapper::new(delegate).wrap())
    }
}

/// Translates CEF -> Rust callbacks.
struct DOMVisitorWrapper(Box<dyn DOMVisitorCallbacks>);

impl DOMVisitorWrapper {
    pub fn new<C: DOMVisitorCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }
    unsafe extern "C" fn visit(this: *mut _cef_domvisitor_t, document: *mut _cef_domdocument_t) {
        let this: &mut Self = Wrapped::wrappable(this);
        let document = DOMDocument::from_ptr_unchecked(document);

        this.0.visit(document);
    }
}

impl Wrappable for DOMVisitorWrapper {
    type Cef = cef_domvisitor_t;

    /// Converts this to a smart pointer.
    fn wrap(self) -> RefCountedPtr<cef_domvisitor_t> {
        RefCountedPtr::wrap(
            cef_domvisitor_t {
                base:  unsafe { zeroed() },
                visit: Some(Self::visit)
            },
            self
        )
    }
}

// Structure used to represent a DOM node. The functions of this structure
// should only be called on the render process main thread.
ref_counted_ptr!(DOMNode, cef_domnode_t);

impl DOMNode {
    /// Returns the type for this node.
    pub fn get_type(&self) -> Result<DOMNodeType> {
        try_c!(self, get_type, { Ok(get_type(self.as_ptr()).into()) })
    }

    /// Returns true if this is a text node.
    pub fn is_text(&self) -> Result<bool> {
        try_c!(self, is_text, { Ok(is_text(self.as_ptr()) != 0) })
    }

    /// Returns true if this is an element node.
    pub fn is_element(&self) -> Result<bool> {
        try_c!(self, is_element, { Ok(is_element(self.as_ptr()) != 0) })
    }

    /// Returns true if this is an editable node.
    pub fn is_editable(&self) -> Result<bool> {
        try_c!(self, is_editable, { Ok(is_editable(self.as_ptr()) != 0) })
    }

    /// Returns true if this is a form control element node.
    pub fn is_form_control_element(&self) -> Result<bool> {
        try_c!(self, is_form_control_element, {
            Ok(is_form_control_element(self.as_ptr()) != 0)
        })
    }

    /// Returns the type of this form control element node.
    pub fn get_form_control_element_type(&self) -> Result<DOMFormControlType> {
        try_c!(self, get_form_control_element_type, {
            Ok(get_form_control_element_type(self.as_ptr()).into())
        })
    }

    /// Returns true if this object is pointing to the same handle as |that|
    /// object.
    pub fn is_same(&self, that: &DOMNode) -> Result<bool> {
        try_c!(self, is_same, {
            Ok(is_same(self.as_ptr(), that.as_ptr()) != 0)
        })
    }

    /// Returns the name of this node.
    pub fn get_name(&self) -> Result<String> {
        try_c!(self, get_name, {
            let s = get_name(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns the value of this node.
    pub fn get_value(&self) -> Result<String> {
        try_c!(self, get_value, {
            let s = get_value(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Set the value of this node. Returns true on success.
    pub fn set_value(&self, value: &str) -> Result<bool> {
        try_c!(self, set_value, {
            Ok(set_value(self.as_ptr(), CefString::from(value).as_ptr()) != 0)
        })
    }

    /// Returns the contents of this node as markup.
    pub fn get_as_markup(&self) -> Result<String> {
        try_c!(self, get_as_markup, {
            let s = get_as_markup(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns the document associated with this node.
    pub fn get_document(&self) -> Result<DOMDocument> {
        try_c!(self, get_document, {
            Ok(DOMDocument::from_ptr_unchecked(get_document(self.as_ptr())))
        })
    }

    /// Returns the parent node.
    pub fn get_parent(&self) -> Result<DOMNode> {
        try_c!(self, get_parent, {
            Ok(DOMNode::from_ptr_unchecked(get_parent(self.as_ptr())))
        })
    }

    /// Returns the previous sibling node.
    pub fn get_previous_sibling(&self) -> Result<DOMNode> {
        try_c!(self, get_previous_sibling, {
            Ok(DOMNode::from_ptr_unchecked(get_previous_sibling(
                self.as_ptr()
            )))
        })
    }

    /// Returns the next sibling node.
    pub fn get_next_sibling(&self) -> Result<DOMNode> {
        try_c!(self, get_next_sibling, {
            Ok(DOMNode::from_ptr_unchecked(get_next_sibling(self.as_ptr())))
        })
    }

    /// Returns true if this node has child nodes.
    pub fn has_children(&self) -> Result<bool> {
        try_c!(self, has_children, { Ok(has_children(self.as_ptr()) != 0) })
    }

    /// Return the first child node.
    pub fn get_first_child(&self) -> Result<DOMNode> {
        try_c!(self, get_first_child, {
            Ok(DOMNode::from_ptr_unchecked(get_first_child(self.as_ptr())))
        })
    }

    /// Returns the last child node.
    pub fn get_last_child(&self) -> Result<DOMNode> {
        try_c!(self, get_last_child, {
            Ok(DOMNode::from_ptr_unchecked(get_last_child(self.as_ptr())))
        })
    }

    /// Returns the tag name of this element.
    pub fn get_element_tag_name(&self) -> Result<String> {
        try_c!(self, get_element_tag_name, {
            let s = get_element_tag_name(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns true if this element has attributes.
    pub fn has_element_attributes(&self) -> Result<bool> {
        try_c!(self, has_element_attributes, {
            Ok(has_element_attributes(self.as_ptr()) != 0)
        })
    }

    /// Returns true if this element has an attribute named |attrName|.
    pub fn has_element_attribute(&self, attr_name: &str) -> Result<bool> {
        try_c!(self, has_element_attribute, {
            Ok(has_element_attribute(self.as_ptr(), CefString::from(attr_name).as_ptr()) != 0)
        })
    }

    /// Returns the element attribute named |attrName|.
    pub fn get_element_attribute(&self, attr_name: &str) -> Result<String> {
        try_c!(self, get_element_attribute, {
            let s = get_element_attribute(self.as_ptr(), CefString::from(attr_name).as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns a map of all element attributes.
    pub fn get_element_attributes(&self) -> Result<HashMap<String, String>> {
        try_c!(self, get_element_attributes, {
            let mut attr_map = CefStringMap::new();
            get_element_attributes(self.as_ptr(), attr_map.as_mut_ptr());

            Ok(attr_map.into())
        })
    }

    /// Set the value for the element attribute named |attrName|. Returns true on
    /// success.
    pub fn set_element_attribute(&self, attr_name: &str, value: &str) -> Result<bool> {
        try_c!(self, set_element_attribute, {
            Ok(set_element_attribute(
                self.as_ptr(),
                CefString::from(attr_name).as_ptr(),
                CefString::from(value).as_ptr()
            ) != 0)
        })
    }

    /// Returns the inner text of the element.
    pub fn get_element_inner_text(&self) -> Result<String> {
        try_c!(self, get_element_inner_text, {
            let s = get_element_inner_text(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns the bounds of the element in device pixels. Use
    /// "window.devicePixelRatio" to convert to/from CSS pixels.
    pub fn get_element_bounds(&self) -> Result<Rect> {
        try_c!(self, get_element_bounds, {
            Ok(get_element_bounds(self.as_ptr()).into())
        })
    }
}

// Structure used to represent a DOM document. The functions of this structure
// should only be called on the render process main thread thread.
ref_counted_ptr!(DOMDocument, cef_domdocument_t);

impl DOMDocument {
    /// Returns the document type.
    pub fn get_type(&self) -> Result<DOMDocumentType> {
        try_c!(self, get_type, { Ok(get_type(self.as_ptr()).into()) })
    }

    /// Returns the root document node.
    pub fn get_document(&self) -> Result<DOMNode> {
        try_c!(self, get_document, {
            Ok(DOMNode::from_ptr_unchecked(get_document(self.as_ptr())))
        })
    }

    /// Returns the BODY node of an HTML document.
    pub fn get_body(&self) -> Result<DOMNode> {
        try_c!(self, get_body, {
            Ok(DOMNode::from_ptr_unchecked(get_body(self.as_ptr())))
        })
    }

    /// Returns the HEAD node of an HTML document.
    pub fn get_head(&self) -> Result<DOMNode> {
        try_c!(self, get_head, {
            Ok(DOMNode::from_ptr_unchecked(get_head(self.as_ptr())))
        })
    }

    /// Returns the title of an HTML document.
    pub fn get_title(&self) -> Result<String> {
        try_c!(self, get_title, {
            let s = get_title(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns the document element with the specified ID value.
    pub fn get_element_by_id(&self, id: &str) -> Result<DOMNode> {
        try_c!(self, get_element_by_id, {
            let e = get_element_by_id(self.as_ptr(), CefString::from(id).as_ptr());
            Ok(DOMNode::from_ptr_unchecked(e))
        })
    }

    /// Returns the node that currently has keyboard focus.
    pub fn get_focused_node(&self) -> Result<DOMNode> {
        try_c!(self, get_focused_node, {
            Ok(DOMNode::from_ptr_unchecked(get_focused_node(self.as_ptr())))
        })
    }

    /// Returns true if a portion of the document is selected.
    pub fn has_selection(&self) -> Result<bool> {
        try_c!(self, has_selection, {
            Ok(has_selection(self.as_ptr()) != 0)
        })
    }

    /// Returns the selection offset within the start node.
    pub fn get_selection_start_offset(&self) -> Result<i32> {
        try_c!(self, get_selection_start_offset, {
            Ok(get_selection_start_offset(self.as_ptr()))
        })
    }

    /// Returns the selection offset within the end node.
    pub fn get_selection_end_offset(&self) -> Result<i32> {
        try_c!(self, get_selection_end_offset, {
            Ok(get_selection_end_offset(self.as_ptr()))
        })
    }

    /// Returns the contents of this selection as markup.
    pub fn get_selection_as_markup(&self) -> Result<String> {
        try_c!(self, get_selection_as_markup, {
            let s = get_selection_as_markup(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns the contents of this selection as text.
    pub fn get_selection_as_text(&self) -> Result<String> {
        try_c!(self, get_selection_as_text, {
            let s = get_selection_as_text(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns the base URL for the document.
    pub fn get_base_url(&self) -> Result<String> {
        try_c!(self, get_base_url, {
            let s = get_base_url(self.as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }

    /// Returns a complete URL based on the document base URL and the specified
    /// partial URL.
    pub fn get_complete_url(&self, partial_url: &str) -> Result<String> {
        try_c!(self, get_complete_url, {
            let s = get_complete_url(self.as_ptr(), CefString::from(partial_url).as_ptr());
            Ok(CefString::from_userfree_ptr_unchecked(s).into())
        })
    }
}
