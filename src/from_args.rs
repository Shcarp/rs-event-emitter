use crate::impl_from_args;
use crate::types::ArcAny;
use emitter_all_tuples::all_tuples;

pub trait FromArgs: Sized {
    fn from_args(args: &[ArcAny]) -> Option<Self>;
}

all_tuples!(impl_from_args, 16, P);
