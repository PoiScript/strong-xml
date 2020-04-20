use proc_macro2::Span;
use syn::GenericArgument;
use syn::PathArguments;
use syn::{Lifetime, Type};

// replace every lifetime with an anonymous lifetime
//
// Foo<'bar> => Foo<'_>
// Foo => Foo
pub fn elide_type_lifetimes(ty: &mut Type) {
    if let Type::Path(ty) = ty {
        ty.path.segments.iter_mut().for_each(|seg| {
            if let PathArguments::AngleBracketed(args) = &mut seg.arguments {
                args.args.iter_mut().for_each(|arg| {
                    if let GenericArgument::Lifetime(lt) = arg {
                        *lt = Lifetime::new("'_", Span::call_site())
                    }
                });
            }
        })
    }
    // TODO: should we take care of other types?
}
