use proc_macro2::Span;
use syn::token::Colon;
use syn::{Generics, Lifetime, LifetimeDef};

// generate a new lifetime ('__input) which outlives every lifetimes from the given generics
//
// <'a, 'b, T> => '__input: 'a + 'b
// <T> => '__input
pub fn gen_input_lifetime(generics: &Generics) -> LifetimeDef {
    let lt = Lifetime::new("'__input", Span::call_site());

    if generics.lifetimes().count() == 0 {
        return LifetimeDef::new(lt);
    }

    LifetimeDef {
        attrs: Vec::new(),
        lifetime: lt,
        colon_token: Some(Colon::default()),
        bounds: generics.lifetimes().map(|lt| lt.lifetime.clone()).collect(),
    }
}
