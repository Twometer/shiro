use self::{heap::Heap, native::NativeLibProvider};

pub mod eval;
pub mod heap;
mod native;
mod preproc;
pub mod scope;
pub mod value;

pub struct RunContext<'a> {
    pub heap: &'a mut Heap,
    pub libs: &'a NativeLibProvider,
}

impl<'a> RunContext<'a> {
    fn new(heap: &'a mut Heap, libs: &'a NativeLibProvider) -> RunContext<'a> {
        RunContext { heap, libs }
    }
}
