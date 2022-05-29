use super::method_signature::RequestHandlerSignature;
use crate::utils::enum_variant::EnumVariant;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

#[derive(Debug, FromMeta)]
pub struct MacroArgs {
    cmd: String,
    normal: u16,
    translate: u16,
}

fn get_server_impl(protocol_method_fn_def: &ItemFn, args: &MacroArgs) -> proc_macro2::TokenStream {
    let request_handler_ident = &protocol_method_fn_def.sig.ident;
    let MacroArgs {
        cmd: command,
        normal: normal_params,
        translate: translate_params,
    } = args;
    let command_stream: proc_macro2::TokenStream = command.parse().unwrap();
    let command_enum = syn::parse2::<EnumVariant>(command_stream).unwrap();
    let command_token = command_enum.token();
    let service_ident = &command_enum.ident;

    let RequestHandlerSignature { server, input } =
        RequestHandlerSignature::new(protocol_method_fn_def);

    let input_read = match input {
        Some(_) => quote! {
            let input = ctr::ipc::Command::read()?.into_data();
            let raw_out = #request_handler_ident(self, session_index, input)?;
        },
        None => quote! {
            let raw_out = #request_handler_ident(self, session_index)?;
        },
    };

    quote! {
        impl ctr::sysmodule::server::ServiceRoute<#service_ident, { #command_token as u16 }> for #server {
            fn handle_request(&mut self, session_index: usize) -> ctr::result::CtrResult<ctr::ipc::WrittenCommand> {
                #input_read
                let out = ctr::sysmodule::server::CtrSuccessResponse::new(raw_out);
                let written = ctr::ipc::Command::new_from_parts(#command_token, #normal_params, #translate_params, out).write();
                Ok(written)
            }
        }
    }
}

pub fn impl_ctr_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as AttributeArgs);
    let protocol_method_fn_def = parse_macro_input!(item as ItemFn);

    let args = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let server_impl = get_server_impl(&protocol_method_fn_def, &args);

    quote! {
        #protocol_method_fn_def

        #server_impl
    }
    .into()
}
