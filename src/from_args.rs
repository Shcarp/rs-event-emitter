use crate::impl_from_args;
use crate::types::Param;
use emitter_all_tuples::all_tuples;

pub trait FromArgs: Sized {
    fn from_args(args: &[Param]) -> Option<Self>;
}

all_tuples!(impl_from_args, 16, P);
