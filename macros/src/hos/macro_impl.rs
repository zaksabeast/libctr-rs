use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn impl_hos(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func_def = parse_macro_input!(item as ItemFn);
    let func_sig = &func_def.sig;
    let func_vis = &func_def.vis;

    quote! {
        #[cfg(target_os = "horizon")]
        #func_def

        #[cfg(not(target_os = "horizon"))]
        #[allow(unused)]
        #func_vis #func_sig {
          unimplemented!("Only available on HOS")
        }

    }
    .into()
}
