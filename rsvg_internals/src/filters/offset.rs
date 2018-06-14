use std::cell::Cell;

use cairo::{self, ImageSurface};

use attributes::Attribute;
use error::NodeError;
use handle::RsvgHandle;
use node::{NodeResult, NodeTrait, RsvgCNodeImpl, RsvgNode};
use parsers;
use property_bag::PropertyBag;
use util::clamp;

use super::context::{FilterContext, FilterOutput, FilterResult, IRect};
use super::iterators::{ImageSurfaceDataExt, ImageSurfaceDataShared, Pixels};
use super::{make_result, Filter, FilterError, PrimitiveWithInput};

/// The `feOffset` filter primitive.
pub struct Offset {
    base: PrimitiveWithInput,
    dx: Cell<f64>,
    dy: Cell<f64>,
}

impl Offset {
    /// Constructs a new `Offset` with empty properties.
    #[inline]
    pub fn new() -> Offset {
        Offset {
            base: PrimitiveWithInput::new::<Self>(),
            dx: Cell::new(0f64),
            dy: Cell::new(0f64),
        }
    }
}

impl NodeTrait for Offset {
    fn set_atts(
        &self,
        node: &RsvgNode,
        handle: *const RsvgHandle,
        pbag: &PropertyBag,
    ) -> NodeResult {
        self.base.set_atts(node, handle, pbag)?;

        for (_key, attr, value) in pbag.iter() {
            match attr {
                Attribute::Dx => self
                    .dx
                    .set(parsers::number(value).map_err(|err| NodeError::parse_error(attr, err))?),
                Attribute::Dy => self
                    .dy
                    .set(parsers::number(value).map_err(|err| NodeError::parse_error(attr, err))?),
                _ => (),
            }
        }

        Ok(())
    }

    #[inline]
    fn get_c_impl(&self) -> *const RsvgCNodeImpl {
        self.base.get_c_impl()
    }
}

impl Filter for Offset {
    fn render(&self, _node: &RsvgNode, ctx: &FilterContext) -> Result<FilterResult, FilterError> {
        let input = make_result(self.base.get_input(ctx))?;
        let bounds = self.base.get_bounds(ctx).add_input(&input).into_irect();

        let dx = self.dx.get();
        let dy = self.dy.get();
        let paffine = ctx.paffine();
        let ox = (paffine.xx * dx + paffine.xy * dy) as i32;
        let oy = (paffine.yx * dx + paffine.yy * dy) as i32;

        let input_data = unsafe {
            ImageSurfaceDataShared::new_unchecked(input.surface())
                .map_err(FilterError::BadInputSurfaceStatus)?
        };

        // input_bounds contains all pixels within bounds,
        // for which (x + ox) and (y + oy) also lie within bounds.
        let input_bounds = IRect {
            x0: clamp(bounds.x0 - ox, bounds.x0, bounds.x1),
            y0: clamp(bounds.y0 - oy, bounds.y0, bounds.y1),
            x1: clamp(bounds.x1 - ox, bounds.x0, bounds.x1),
            y1: clamp(bounds.y1 - oy, bounds.y0, bounds.y1),
        };

        let mut output_surface = ImageSurface::create(
            cairo::Format::ARgb32,
            input_data.width as i32,
            input_data.height as i32,
        ).map_err(FilterError::OutputSurfaceCreation)?;

        let output_stride = output_surface.get_stride() as usize;
        {
            let mut output_data = output_surface.get_data().unwrap();

            for (x, y, pixel) in Pixels::new(input_data, input_bounds) {
                let output_x = (x as i32 + ox) as usize;
                let output_y = (y as i32 + oy) as usize;
                output_data.set_pixel(output_stride, pixel, output_x, output_y);
            }
        }

        Ok(FilterResult {
            name: self.base.result.borrow().clone(),
            output: FilterOutput {
                surface: output_surface,
                bounds,
            },
        })
    }
}
