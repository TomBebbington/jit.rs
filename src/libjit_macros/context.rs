use syntax::ast::{Item, P, Ty};
use syntax::codemap::Span;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
/// A context for a macro to run on
#[deriving(Copy)]
pub struct MacroContext<'a, 'b> {
    pub cx: &'a ExtCtxt<'b>,
    /// The position of the `#[jit_compile]`
    pub pos: Span,
    /// The item the `#[jit_compile]` is before
    pub item: P<Item>
}
impl<'a, 'b> MacroContext<'a, 'b> {
    pub fn new(cx:&'a ExtCtxt<'b>, pos:Span, item:P<Item>) -> MacroContext<'a, 'b> {
        MacroContext {
            cx: cx,
            pos: pos,
            item: item
        }
    }
    #[inline(always)]
    pub fn get_curr(&self) -> P<Ty> {
        self.cx.ty_ident(self.pos, self.item.ident)
    }
}