use quote::quote;
use std::fmt;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Result, Token, Variant,
};

#[derive(Debug)]
pub struct EnumVariant {
    pub ident: Ident,
    _colon: Token![::],
    pub variant: Variant,
}

impl fmt::Display for EnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.ident, self.variant.ident)
    }
}

impl EnumVariant {
    pub fn token(&self) -> proc_macro2::TokenStream {
        let EnumVariant { ident, variant, .. } = self;
        quote! {
            #ident::#variant
        }
    }
}

impl Parse for EnumVariant {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            _colon: input.parse()?,
            variant: input.parse()?,
        })
    }
}
