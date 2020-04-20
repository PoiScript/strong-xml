use proc_macro2::TokenStream;
use quote::quote;

use crate::types::Type;

pub fn read(ty: &Type, ele_name: TokenStream) -> TokenStream {
    let ty = match ty {
        Type::T(ty) => ty,
        _ => panic!("strong-xml only supports newtype_struct and newtype_enum for now."),
    };

    quote! {
        strong_xml::log_start_reading!(#ele_name);

        let res = <#ty as XmlRead>::from_reader(reader)?;

        strong_xml::log_finish_reading!(#ele_name);

        return Ok(#ele_name(res));
    }
}
