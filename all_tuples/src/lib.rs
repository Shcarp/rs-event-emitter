use proc_macro::TokenStream;
use quote::{
    format_ident, 
    quote
};
use syn::{
    parse::{Parse, ParseStream, Result}, 
    parse_macro_input, 
    token::Comma, 
    Ident, 
    LitInt
};

// 描述宏需要解析到的数据
struct AllTuples {
    macro_ident: Ident,
    len: usize,
    ident: Ident,
}

impl Parse for AllTuples {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let len = input.parse::<LitInt>()?.base10_parse()?; 
        input.parse::<Comma>()?;
        let ident =input.parse::<Ident>()?;

        Ok(AllTuples {
            macro_ident,
            len,
            ident,
        })
    }
}

#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AllTuples);
    let mut ident_tuples = Vec::with_capacity(input.len);
    for i in 0..input.len - 1{
        let ident = format_ident!("{}{}", input.ident, i);
        ident_tuples.push(quote! {
            #ident
        });
    }

    let macro_ident = &input.macro_ident;
    let invocations = (0..=input.len - 1).map(|i| {
        let ident_tuples = &ident_tuples[..i];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });
    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}
