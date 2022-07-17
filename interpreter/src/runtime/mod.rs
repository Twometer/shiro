use crate::parser::ShiroParser;

use self::{heap::Heap, native::NativeLibProvider};

pub mod eval;
pub mod heap;
mod native;
pub mod scope;
pub mod value;

pub struct RunContext<'a> {
    pub heap: &'a mut Heap,
    pub libs: &'a NativeLibProvider,
    pub parser: ShiroParser,
}

impl<'a> RunContext<'a> {
    fn new(heap: &'a mut Heap, libs: &'a NativeLibProvider) -> RunContext<'a> {
        RunContext {
            heap,
            libs,
            parser: ShiroParser::new(),
        }
    }
}
