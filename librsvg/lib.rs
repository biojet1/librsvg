#![allow(clippy::missing_safety_doc)]
#![warn(unused)]

#[rustfmt::skip]
pub use crate::c_api::{
    rsvg_error_get_type,
    rsvg_handle_close,
    rsvg_handle_flags_get_type,
    rsvg_handle_get_base_uri,
    rsvg_handle_get_dimensions,
    rsvg_handle_get_dimensions_sub,
    rsvg_handle_get_geometry_for_element,
    rsvg_handle_get_geometry_for_layer,
    rsvg_handle_get_intrinsic_dimensions,
    rsvg_handle_get_pixbuf_sub,
    rsvg_handle_get_position_sub,
    rsvg_handle_has_sub,
    rsvg_handle_internal_set_testing,
    rsvg_handle_new_from_data,
    rsvg_handle_new_from_file,
    rsvg_handle_new_from_gfile_sync,
    rsvg_handle_new_from_stream_sync,
    rsvg_handle_new_with_flags,
    rsvg_handle_read_stream_sync,
    rsvg_handle_render_cairo_sub,
    rsvg_handle_render_element,
    rsvg_handle_render_document,
    rsvg_handle_render_layer,
    rsvg_handle_set_base_gfile,
    rsvg_handle_set_base_uri,
    rsvg_handle_set_dpi_x_y,
    rsvg_handle_set_size_callback,
    rsvg_handle_write,
};

pub use crate::color_utils::rsvg_css_parse_color;

pub use crate::dpi::{rsvg_set_default_dpi, rsvg_set_default_dpi_x_y};

#[rustfmt::skip]
pub use crate::pixbuf_utils::{
    rsvg_pixbuf_from_file,
    rsvg_pixbuf_from_file_at_max_size,
    rsvg_pixbuf_from_file_at_size,
    rsvg_pixbuf_from_file_at_zoom,
    rsvg_pixbuf_from_file_at_zoom_with_max,
};

#[macro_use]
mod messages;

mod c_api;
mod color_utils;
mod dpi;
pub mod pixbuf_utils;
