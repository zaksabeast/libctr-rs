use proc_macro::TokenStream;

mod ctr_method;
mod ctr_start;
mod hos;
mod match_ctr_route;
mod utils;

#[proc_macro_attribute]
pub fn ctr_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    ctr_method::impl_ctr_method(attr, item)
}

#[proc_macro_attribute]
pub fn ctr_start(attr: TokenStream, item: TokenStream) -> TokenStream {
    ctr_start::impl_ctr_start(attr, item)
}

#[proc_macro_attribute]
pub fn hos(attr: TokenStream, item: TokenStream) -> TokenStream {
    hos::impl_hos(attr, item)
}

#[proc_macro]
pub fn match_ctr_route(item: TokenStream) -> TokenStream {
    match_ctr_route::impl_match_ctr_route(item)
}
