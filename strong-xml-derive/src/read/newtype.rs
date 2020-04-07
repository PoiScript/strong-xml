use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::types::Type;

pub fn read(ty: &Type, ele_name: TokenStream) -> TokenStream {
    let ty = match ty {
        Type::T(ty) => {
            if let Some(ty) = trim_lifetime(ty) {
                quote!(#ty)
            } else {
                quote!(#ty)
            }
        }
        _ => panic!("strong-xml only supports newtype_struct and newtype_enum for now."),
    };

    quote! {
        log::debug!(concat!("[", stringify!(#ele_name), "] Started reading"));

        let res = #ty::from_reader(reader)?;

        log::debug!(concat!("[", stringify!(#ele_name), "] Finished reading"));

        return Ok(#ele_name(res));
    }
}

fn trim_lifetime(ty: &syn::Type) -> Option<&Ident> {
    let path = match ty {
        syn::Type::Path(ty) => &ty.path,
        _ => return None,
    };
    let seg = path.segments.last()?;
    Some(&seg.ident)
}
