/* -*- Mode: C; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/* vim: set sw=4 sts=4 expandtab: */

#ifndef RSVG_ATTRIBUTES_H
#define RSVG_ATTRIBUTES_H

#include <glib.h>

/* Keep this in sync with rust/src/build.rs */
typedef enum {
    RSVG_ATTRIBUTE_ALTERNATE,
    RSVG_ATTRIBUTE_AMPLITUDE,
    RSVG_ATTRIBUTE_AZIMUTH,
    RSVG_ATTRIBUTE_BASE_FREQUENCY,
    RSVG_ATTRIBUTE_BASELINE_SHIFT,
    RSVG_ATTRIBUTE_BIAS,
    RSVG_ATTRIBUTE_CLASS,
    RSVG_ATTRIBUTE_CLIP_PATH,
    RSVG_ATTRIBUTE_CLIP_RULE,
    RSVG_ATTRIBUTE_CLIP_PATH_UNITS,
    RSVG_ATTRIBUTE_COLOR,
    RSVG_ATTRIBUTE_COMP_OP,
    RSVG_ATTRIBUTE_CX,
    RSVG_ATTRIBUTE_CY,
    RSVG_ATTRIBUTE_D,
    RSVG_ATTRIBUTE_DIFFUSE_CONSTANT,
    RSVG_ATTRIBUTE_DIRECTION,
    RSVG_ATTRIBUTE_DISPLAY,
    RSVG_ATTRIBUTE_DIVISOR,
    RSVG_ATTRIBUTE_DX,
    RSVG_ATTRIBUTE_DY,
    RSVG_ATTRIBUTE_EDGE_MODE,
    RSVG_ATTRIBUTE_ELEVATION,
    RSVG_ATTRIBUTE_ENABLE_BACKGROUND,
    RSVG_ATTRIBUTE_ENCODING,
    RSVG_ATTRIBUTE_EXPONENT,
    RSVG_ATTRIBUTE_FILL,
    RSVG_ATTRIBUTE_FILL_OPACITY,
    RSVG_ATTRIBUTE_FILL_RULE,
    RSVG_ATTRIBUTE_FILTER,
    RSVG_ATTRIBUTE_FILTER_UNITS,
    RSVG_ATTRIBUTE_FLOOD_COLOR,
    RSVG_ATTRIBUTE_FLOOD_OPACITY,
    RSVG_ATTRIBUTE_FONT_FAMILY,
    RSVG_ATTRIBUTE_FONT_SIZE,
    RSVG_ATTRIBUTE_FONT_STRETCH,
    RSVG_ATTRIBUTE_FONT_STYLE,
    RSVG_ATTRIBUTE_FONT_VARIANT,
    RSVG_ATTRIBUTE_FONT_WEIGHT,
    RSVG_ATTRIBUTE_FX,
    RSVG_ATTRIBUTE_FY,
    RSVG_ATTRIBUTE_GRADIENT_TRANSFORM,
    RSVG_ATTRIBUTE_GRADIENT_UNITS,
    RSVG_ATTRIBUTE_HEIGHT,
    RSVG_ATTRIBUTE_HREF,
    RSVG_ATTRIBUTE_ID,
    RSVG_ATTRIBUTE_IN,
    RSVG_ATTRIBUTE_IN2,
    RSVG_ATTRIBUTE_INTERCEPT,
    RSVG_ATTRIBUTE_K1,
    RSVG_ATTRIBUTE_K2,
    RSVG_ATTRIBUTE_K3,
    RSVG_ATTRIBUTE_K4,
    RSVG_ATTRIBUTE_KERNEL_MATRIX,
    RSVG_ATTRIBUTE_KERNEL_UNIT_LENGTH,
    RSVG_ATTRIBUTE_LETTER_SPACING,
    RSVG_ATTRIBUTE_LIGHTING_COLOR,
    RSVG_ATTRIBUTE_LIMITING_CONE_ANGLE,
    RSVG_ATTRIBUTE_MARKER,
    RSVG_ATTRIBUTE_MARKER_END,
    RSVG_ATTRIBUTE_MARKER_MID,
    RSVG_ATTRIBUTE_MARKER_START,
    RSVG_ATTRIBUTE_MARKER_HEIGHT,
    RSVG_ATTRIBUTE_MARKER_UNITS,
    RSVG_ATTRIBUTE_MARKER_WIDTH,
    RSVG_ATTRIBUTE_MASK,
    RSVG_ATTRIBUTE_MASK_CONTENT_UNITS,
    RSVG_ATTRIBUTE_MASK_UNITS,
    RSVG_ATTRIBUTE_MODE,
    RSVG_ATTRIBUTE_NUM_OCTAVES,
    RSVG_ATTRIBUTE_OFFSET,
    RSVG_ATTRIBUTE_OPACITY,
    RSVG_ATTRIBUTE_OPERATOR,
    RSVG_ATTRIBUTE_ORDER,
    RSVG_ATTRIBUTE_ORIENT,
    RSVG_ATTRIBUTE_OVERFLOW,
    RSVG_ATTRIBUTE_PARSE,
    RSVG_ATTRIBUTE_PATH,
    RSVG_ATTRIBUTE_PATTERN_CONTENT_UNITS,
    RSVG_ATTRIBUTE_PATTERN_TRANSFORM,
    RSVG_ATTRIBUTE_PATTERN_UNITS,
    RSVG_ATTRIBUTE_POINTS,
    RSVG_ATTRIBUTE_POINTS_AT_X,
    RSVG_ATTRIBUTE_POINTS_AT_Y,
    RSVG_ATTRIBUTE_POINTS_AT_Z,
    RSVG_ATTRIBUTE_PRESERVE_ALPHA,
    RSVG_ATTRIBUTE_PRESERVE_ASPECT_RATIO,
    RSVG_ATTRIBUTE_PRIMITIVE_UNITS,
    RSVG_ATTRIBUTE_R,
    RSVG_ATTRIBUTE_RADIUS,
    RSVG_ATTRIBUTE_REF_X,
    RSVG_ATTRIBUTE_REF_Y,
    RSVG_ATTRIBUTE_REQUIRED_EXTENSIONS,
    RSVG_ATTRIBUTE_REQUIRED_FEATURES,
    RSVG_ATTRIBUTE_RESULT,
    RSVG_ATTRIBUTE_RX,
    RSVG_ATTRIBUTE_RY,
    RSVG_ATTRIBUTE_SCALE,
    RSVG_ATTRIBUTE_SEED,
    RSVG_ATTRIBUTE_SHAPE_RENDERING,
    RSVG_ATTRIBUTE_SLOPE,
    RSVG_ATTRIBUTE_SPECULAR_CONSTANT,
    RSVG_ATTRIBUTE_SPECULAR_EXPONENT,
    RSVG_ATTRIBUTE_SPREAD_METHOD,
    RSVG_ATTRIBUTE_STD_DEVIATION,
    RSVG_ATTRIBUTE_STITCH_TILES,
    RSVG_ATTRIBUTE_STOP_COLOR,
    RSVG_ATTRIBUTE_STOP_OPACITY,
    RSVG_ATTRIBUTE_STROKE,
    RSVG_ATTRIBUTE_STROKE_DASHARRAY,
    RSVG_ATTRIBUTE_STROKE_DASHOFFSET,
    RSVG_ATTRIBUTE_STROKE_LINECAP,
    RSVG_ATTRIBUTE_STROKE_LINEJOIN,
    RSVG_ATTRIBUTE_STROKE_MITERLIMIT,
    RSVG_ATTRIBUTE_STROKE_OPACITY,
    RSVG_ATTRIBUTE_STROKE_WIDTH,
    RSVG_ATTRIBUTE_STYLE,
    RSVG_ATTRIBUTE_SURFACE_SCALE,
    RSVG_ATTRIBUTE_SYSTEM_LANGUAGE,
    RSVG_ATTRIBUTE_TABLE_VALUES,
    RSVG_ATTRIBUTE_TARGET_X,
    RSVG_ATTRIBUTE_TARGET_Y,
    RSVG_ATTRIBUTE_TEXT_ANCHOR,
    RSVG_ATTRIBUTE_TEXT_DECORATION,
    RSVG_ATTRIBUTE_TEXT_RENDERING,
    RSVG_ATTRIBUTE_TRANSFORM,
    RSVG_ATTRIBUTE_TYPE,
    RSVG_ATTRIBUTE_UNICODE_BIDI,
    RSVG_ATTRIBUTE_VALUES,
    RSVG_ATTRIBUTE_VERTS,
    RSVG_ATTRIBUTE_VIEW_BOX,
    RSVG_ATTRIBUTE_VISIBILITY,
    RSVG_ATTRIBUTE_WIDTH,
    RSVG_ATTRIBUTE_WRITING_MODE,
    RSVG_ATTRIBUTE_X,
    RSVG_ATTRIBUTE_X1,
    RSVG_ATTRIBUTE_Y1,
    RSVG_ATTRIBUTE_X2,
    RSVG_ATTRIBUTE_Y2,
    RSVG_ATTRIBUTE_X_CHANNEL_SELECTOR,
    RSVG_ATTRIBUTE_XLINK_HREF,
    RSVG_ATTRIBUTE_XML_LANG,
    RSVG_ATTRIBUTE_XML_SPACE,
    RSVG_ATTRIBUTE_Y,
    RSVG_ATTRIBUTE_Y_CHANNEL_SELECTOR,
    RSVG_ATTRIBUTE_Z,
} RsvgAttribute;

/* Implemented in rust/src/attributes.rs */
G_GNUC_INTERNAL
gboolean rsvg_attribute_from_name (const char *name, RsvgAttribute *out_attr);

#endif /* RSVG_ATTRIBUTES_H */
