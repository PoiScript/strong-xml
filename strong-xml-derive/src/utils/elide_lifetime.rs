use proc_macro2::Span;
use syn::GenericArgument;
use syn::PathArguments;
use syn::{Lifetime, Type};

// replace every lifetime with an anonymous lifetime
//
// Foo<'bar> => Foo<'_>
// Foo => Foo
pub fn elide_type_lifetimes(ty: &mut Type) {
    match ty {
        Type::Path(ty) => ty
            .path
            .segments
            .iter_mut()
            .for_each(|seg| match &mut seg.arguments {
                PathArguments::AngleBracketed(args) => {
                    args.args.iter_mut().for_each(|arg| match arg {
                        GenericArgument::Lifetime(lt) => {
                            *lt = Lifetime::new("'_", Span::call_site())
                        }
                        _ => (),
                    });
                }
                _ => (),
            }),
        // TODO: should we take care of other types?
        _ => (),
    }
}
