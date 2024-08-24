#[macro_export]
macro_rules! idx_from_args {
    ($ty:ident) => {{
        const TYPE_NAME: &str = stringify!($ty);
        let digits: String = TYPE_NAME
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        if digits.is_empty() {
            panic!(
                "Invalid type name for FromArgs: {}. Type name must end with a number.",
                TYPE_NAME
            );
        }

        match digits.parse::<usize>() {
            Ok(idx) => idx,
            Err(_) => panic!("Failed to parse index from type name: {}", TYPE_NAME),
        }
    }};
}

#[macro_export]
macro_rules! count_tts {
    () => {0};
    ($head:tt $($tail:tt)*) => {1 + $crate::count_tts!($($tail)*)};
}

// 生成 FromArgs 的实现, 参数末尾为从 0 开始的连续数字 比如 P0, P1, P2
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
