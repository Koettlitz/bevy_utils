use std::ops;

// bevy_macro_mirror/src/lib.rs
use proc_macro_crate::{FoundCrate, crate_name};
use quote::ToTokens;
use syn::{Ident, Token, spanned::Spanned};

extern crate self as elf_macro_utils;

pub use mirror_generation::generate_mirror_type;
pub use mirror_transform::*;

mod mirror_generation;
mod mirror_transform;

/// A [`syn::Path`] whose first segment is rewritten to the correct crate name.
///
/// Resolution happens in [`CratePath::try_from`] using
/// [`proc_macro_crate::crate_name`].
#[derive(Clone)]
pub struct CratePath(syn::Path);

impl TryFrom<&str> for CratePath {
    type Error = syn::Error;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        let path: syn::Path = syn::parse_str(path)?;
        Self::try_from(path)
    }
}

impl TryFrom<syn::Path> for CratePath {
    type Error = syn::Error;

    fn try_from(mut path: syn::Path) -> Result<Self, Self::Error> {
        let path_span = path.span();
        let Some(first_segment) = path.segments.first_mut() else {
            return Err(syn::Error::new(path_span, "empty path"));
        };
        let crate_string = first_segment.ident.to_string();
        let span = first_segment.ident.span();

        match crate_name(&crate_string) {
            Ok(FoundCrate::Itself) => {
                // Examples, doctests and integration tests are compiled as
                // separate crates, so `crate::` would be wrong there. Name the
                // crate explicitly; the lib resolves it via
                // `extern crate self as <name>;`.
                path.leading_colon.get_or_insert_with(|| Token![::](span));
            }
            Ok(FoundCrate::Name(name)) => {
                first_segment.ident = Ident::new(&name, span);
            }
            Err(e) => {
                return Err(syn::Error::new(
                    span,
                    format!("could not resolve crate `{crate_string}`: {e}"),
                ));
            }
        }
        Ok(Self(path))
    }
}

impl ops::Deref for CratePath {
    type Target = syn::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<syn::Path> for CratePath {
    fn as_ref(&self) -> &syn::Path {
        &self.0
    }
}

impl From<CratePath> for syn::Path {
    fn from(p: CratePath) -> Self {
        p.0
    }
}

impl ToTokens for CratePath {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}
