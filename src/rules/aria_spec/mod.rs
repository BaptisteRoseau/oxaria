//! WAI-ARIA 1.2 and ARIA in HTML data, and the role, focus and visibility
//! helpers built on it, shared by the `aria12`, `html_aria` and `act` rules.

mod attribute_data;
mod attributes;
mod element;
mod html;
mod role_data;
mod roles;
mod values;

pub use attributes::{ARIA_1_3_ATTRIBUTES, Attribute, Scope, ValueType, attribute, attributes};
pub use element::{
    aria_attributes, effective_role, explicit_roles, first_valid_role, is_aria_hidden, is_disabled,
    is_focusable, is_hidden, is_not_rendered, is_tabbable, tabindex,
};
pub use html::{
    AllowedAria, DEPRECATED_ROLES, NATIVE_EQUIVALENTS, allowed_aria, has_author_name,
    implicit_role, input_type,
};
pub use roles::{NameFrom, Role, concrete_role, role, roles};
pub use values::{
    AriaValue, Parsed, aria_idrefs, aria_number, aria_value, is_aria_true, parse_value,
};
