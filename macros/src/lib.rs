use proc_macro::TokenStream;

mod ctr_method;
mod match_ctr_route;
mod utils;

#[proc_macro_attribute]
pub fn ctr_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    ctr_method::impl_ctr_method(attr, item)
}

#[proc_macro]
pub fn match_ctr_route(item: TokenStream) -> TokenStream {
    match_ctr_route::impl_match_ctr_route(item)
}
