#[macro_export]
macro_rules! idx_from_args {
    ($ty:ident) => {{
        const LOWERCASE: &str = $crate::to_lowercase!($ty);
        let idx = LOWERCASE.as_bytes()[0].wrapping_sub(b'a') as usize;
        if idx >= 26 { panic!("Invalid type name for FromArgs: {}", stringify!($ty)); }
        idx
    }};
}
#[macro_export]
macro_rules! count_tts {
    () => {0};
    ($head:tt $($tail:tt)*) => {1 + $crate::count_tts!($($tail)*)};
}

#[macro_export]
macro_rules! impl_from_args {
    ($($ty:ident),*) => {
        impl<$($ty: 'static + Clone),*> FromArgs for ($($ty,)*) {
            fn from_args(args: &[ArcAny]) -> Option<Self> {
                let expected_len = $crate::count_tts!($($ty)*);
                if args.len() != expected_len {
                    return None;
                }
                Some((
                    $(
                        args.get($crate::idx_from_args!($ty))?.downcast_ref::<$ty>()?.clone(),
                    )*
                ))
            }
        }
    };
}

