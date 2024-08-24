use all_tuples::all_tuples;
use crate::types::ArcAny;
use crate::impl_from_args;

pub trait FromArgs: Sized {
    fn from_args(args: &[ArcAny]) -> Option<Self>;
}

all_tuples!(impl_from_args, 16, P);
