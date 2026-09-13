use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use entity_lookup::lookup_impl;

mod entity_lookup;

#[proc_macro_derive(Lookup, attributes(elf))]
pub fn lookup(item: TokenStream) -> TokenStream {
    match lookup_impl(&parse_macro_input!(item as DeriveInput)) {
        Ok(result) => result.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
