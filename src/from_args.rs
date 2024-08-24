use crate::{impl_from_args, ArcAny};
pub trait FromArgs: Sized {
    fn from_args(args: &[ArcAny]) -> Option<Self>;
}

impl_from_args!(A);
impl_from_args!(A, B);
impl_from_args!(A, B, C);
impl_from_args!(A, B, C, D);
impl_from_args!(A, B, C, D, E);
impl_from_args!(A, B, C, D, E, F);
impl_from_args!(A, B, C, D, E, F, G);
impl_from_args!(A, B, C, D, E, F, G, H);
impl_from_args!(A, B, C, D, E, F, G, H, I);
impl_from_args!(A, B, C, D, E, F, G, H, I, J);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
