use std::borrow::Borrow;
use syn::{FnArg, Ident, ItemFn, Type};

fn get_ident_from_type(ident_type: &Type) -> Option<&Ident> {
    match ident_type {
        Type::Reference(type_ref) => match type_ref.elem.borrow() {
            Type::Path(type_path) => type_path.path.get_ident(),
            _ => None,
        },
        Type::Path(type_path) => type_path.path.get_ident(),
        _ => None,
    }
}

fn get_ident_from_ref_arg(arg: Option<&FnArg>) -> Option<&Ident> {
    if let Some(FnArg::Typed(arg)) = arg {
        return get_ident_from_type(arg.ty.borrow());
    }

    None
}

pub struct RequestHandlerSignature {
    pub server: Ident,
    pub input: Option<Ident>,
}

impl RequestHandlerSignature {
    pub fn new(fn_def: &ItemFn) -> Self {
        let mut args_iter = fn_def.sig.inputs.iter();

        let server = get_ident_from_ref_arg(args_iter.next()).expect("Missing server argument");
        get_ident_from_ref_arg(args_iter.next()).expect("Missing session_index argument");
        let input = get_ident_from_ref_arg(args_iter.next());

        Self {
            server: server.clone(),
            input: input.map(std::clone::Clone::clone),
        }
    }
}
