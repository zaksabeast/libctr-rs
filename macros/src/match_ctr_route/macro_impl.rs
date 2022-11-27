use super::args::Args;
use crate::utils::enum_variant::EnumVariant;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Ident};

fn get_variant_match_branch(
    server: &Ident,
    session_index: &Ident,
    variant: &EnumVariant,
) -> proc_macro2::TokenStream {
    let enum_ident = &variant.ident;
    let variant_token = variant.token();

    quote! {
      #variant_token => <#server as ctr::sysmodule::server::ServiceRoute<
          #enum_ident,
          { #variant_token as u16 },
      >>::handle_request(self, #session_index),
    }
}

pub fn get_ctr_route_branches(
    server: &Ident,
    session_index: &Ident,
    variants: &[&EnumVariant],
) -> proc_macro2::TokenStream {
    let match_branches = variants
        .iter()
        .map(|variant| get_variant_match_branch(server, session_index, variant))
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
      match <ctr::ipc::Command>::current_command_id().into() {
        #(#match_branches)*
        _ => Ok(ctr::ipc::Command::new_from_parts(0u16, 0x1, 0x0, 0xd900182fu32).write()),
      }
    }
}

fn get_unique_enums(args: &Args) -> Vec<Ident> {
    let mut result = args
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect::<Vec<Ident>>();
    result.dedup();
    result
}

fn filter_enum_variants<'a>(
    enum_ident: &Ident,
    variants: &'a Punctuated<EnumVariant, Comma>,
) -> Vec<&'a EnumVariant> {
    variants
        .iter()
        .filter(|variant| variant.ident == *enum_ident)
        .collect()
}

pub fn impl_match_ctr_route(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as Args);
    let service_id = &args.service_id;

    let service_branches = get_unique_enums(&args)
        .iter()
        .map(|enum_ident| {
            let variants = filter_enum_variants(enum_ident, &args.variants);
            let match_branches =
                get_ctr_route_branches(&args.server, &args.session_index, &variants);
            quote! {
              <#enum_ident as ctr::sysmodule::server::Service>::ID => {
                #match_branches
              }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
      match #service_id {
        #(#service_branches)*
        _ => Err(ctr::error::invalid_command()),
      }
    }
    .into()
}
