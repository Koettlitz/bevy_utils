//! This crate loads and resolves assets that reference other assets by name.
//! It builds on [`serde`] and integrates into `bevy`'s asset ecosystem.
//!
//! Hand-written asset types use [`Handle`]s, which aren't serializable. This
//! crate generates a serializable "Def" counterpart for each type, using plain
//! strings in place of [`Handle`]s, along with a [`trait@FromDef`] impl that
//! converts a Def into its runtime type — resolving each string into a
//! [`Handle`] by using [`LoadContext::load()`] along the way.
//!
//! ## Basic usage
//! Let's say you have an animation asset `water_animation.ron`, which references its spritesheet
//! by name:
//! ```ron
//! (
//!     frames: [1, 2, 3,],
//!     frame_duration: (
//!         secs: 0,
//!         nanos: 128000000,
//!     ),
//!     spritesheet: "water",
//! )
//! ```
//! The corresponding struct would look something like this:
//! ```ignore
//! # use bevy_asset::prelude::*;
//! # use bevy_elf::FromDef;
//! # use bevy_reflect::TypePath;
//! # use std::time::Duration;
//!
//! #[derive(FromDef, Asset, TypePath)]
//! struct AnimationAsset {
//!     frames: Vec<usize>,
//!     frame_duration: Duration,
//!     spritesheet: Handle<Spritesheet>,
//! }
//! ```
//! The [`derive@FromDef`] derive macro generates a (de)serializable version of the struct as well as an
//! implementation of the [`trait@FromDef`] trait, which converts it into your struct. The generated
//! struct looks something like this:
//! ```ignore
//! # use std::time::Duration;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize)]
//! struct AnimationDef {
//!     frames: Vec<usize>,
//!     frame_duration: Duration,
//!     spritesheet: String,
//! }
//! ```
//! Assets, that implement [`trait@FromDef`] can be loaded with the [`RonAssetLoader`], which calls
//! [`FromDef::from_def()`] to convert the raw deserialized structure into the runtime
//! structure. You can register the asset and the [`RonAssetLoader`] manually or use the [`AppExt`] extension trait:
//! ```ignore
//! # use bevy_app::prelude::*;
//! # use bevy_elf::AppExt;
//! app.init_ron_asset::<AnimationAsset>().init_ron_asset::<Spritesheet>();
//! ```
//! To resolve the string names into handles some metadata needs to be provided. Let's
//! take the Spritesheet asset as an example:
//! ```ignore
//! # use bevy_asset::prelude::*;
//! # use bevy_elf::{FromDef, asset_spec};
//! #[derive(FromDef, Asset, TypePath)]
//! #[asset_spec(base_path = "spritesheets", extension = "ron")]
//! struct Spritesheet { /* fields omitted */ }
//! ```
//! With the [`asset_spec`] provided the spritesheet handles inside the `AnimationAsset` can now be
//! resolved, e.g. the "water" spritesheet gets resolved into "spritesheets/water.ron".
//!
//! ## Resolving foreign types with `elf`
//! Asset types you don't own cannot be annotated with attributes. Let's take a closer look at the
//! Spritesheet asset:
//! ```ignore
//! # use bevy_asset::prelude::*;
//! # use bevy_elf::{FromDef, asset_spec};
//! # use bevy_image::{Image, TextureAtlasLayout};
//! # use bevy_reflect::TypePath;
//! #[derive(FromDef, Asset, TypePath)]
//! #[asset_spec(base_path = "spritesheets", extension = "ron")]
//! struct Spritesheet {
//!     #[elf(with_spec(base_path = "spritesheets/images", extension = "png"))]
//!     image: Handle<Image>,
//!
//!     #[elf(with_spec(base_path = "spritesheets/layouts", extension = "ron"))]
//!     layout: Handle<TextureAtlasLayout>,
//! }
//! ```
//! Since `Image` and `TextureAtlasLayout` are not defined by you they cannot be resolved
//! the same way, because they don't have an [`asset_spec`].
//! For these you can use the `elf` field attribute to tell `bevy_elf` how to resolve them as shown above.
//!
//! ## Implicit fields
//! It is also possible to make asset references implicit by their name.
//! Imagine your assets are organized in the file system like this:
//! ```text
//! assets/
//! ├── animations/
//! │   ├── water.ron
//! │   └── grass.ron
//! └── spritesheets/
//!     ├── water.ron
//!     ├── grass.ron
//!     ├── images/
//!     │   ├── water.png
//!     │   └── grass.png
//!     └── layouts/
//!         ├── water.ron
//!         └── grass.ron
//! ```
//! Explicitly mentioning e.g. "water" everywhere is cumbersome. Make it implicit instead. The
//! `implicit` flag goes along well with `sub_path`:
//! ```ignore
//! # use bevy_asset::prelude::*;
//! # use bevy_elf::{FromDef, asset_spec};
//! # use bevy_image::{Image, TextureAtlasLayout};
//! # use bevy_reflect::TypePath;
//! #[derive(FromDef, Asset, TypePath)]
//! #[asset_spec(base_path = "spritesheets", extension = "ron")]
//! struct Spritesheet {
//!     #[elf(implicit, with_spec(sub_path = "images", extension = "png"))]
//!     image: Handle<Image>,
//!
//!     #[elf(implicit, with_spec(sub_path = "layouts", extension = "ron"))]
//!     layout: Handle<TextureAtlasLayout>,
//! }
//! ```
//! That way the `image` and `layout` fields are omitted in the generated def type and don't show up
//! in the ron file at all. They are resolved with the same string name as their parent.
//!
//! ## Omitting empty def files
//! With the fields being implicit the spritesheet ron files are now empty. Having to put an empty
//! file there for it all to work isn't nice at all! To omit the whole file tell `bevy_elf` to omit
//! the def type entirely and use `()` instead. Just tell the referencing `AnimationAsset` to not
//! load any file but put a default value into [`FromDef::from_def()`] instead:
//! ```ignore
//! # use bevy_asset::prelude::*;
//! # use std::time::Duration;
//! # use bevy_elf::FromDef;
//! # use bevy_image::{Image, TextureAtlasLayout};
//! # use bevy_reflect::TypePath;
//! #[derive(FromDef, Asset, TypePath)]
//! struct AnimationAsset {
//!     frames: Vec<usize>,
//!     frame_duration: Duration,
//!
//!     #[elf(from_default)]
//!     spritesheet: Handle<Spritesheet>,
//! }
//!
//! #[derive(FromDef)]
//! #[elf(def_type(()))]
//! struct Spritesheet {
//!     #[elf(implicit, with_spec(base_path = "images", extension = "png"))]
//!     image: Handle<Image>,
//!
//!     #[elf(implicit, with_spec(base_path = "layouts", extension = "ron"))]
//!     layout: Handle<TextureAtlasLayout>,
//! }
//! ```
//! That way your directory structure, as well as the ron files themselves become leaner and
//! the spritesheet doesn't appear in the file system at all anymore.
//! But the image and its layout are still neatly stored in their own `Spritesheet` struct.
//! ```text
//! assets/
//! ├── animations/
//! │   ├── water.ron
//! │   └── grass.ron
//! ├── images/
//! │   ├── water.png
//! │   └── grass.png
//! └── layouts/
//!     ├── water.ron
//!     └── grass.ron
//! ```
//! Note that since the spritesheets directory doesn't exist anymore, `images/` and `layouts/`
//! move up to become top-level asset folders, so we changed from `sub_path` back to `base_path`.
//! Also note, that Spritesheet is no `Asset` anymore, since it doesn't get loaded from a file,
//! so the registration via `app.init_ron_asset::<Spritesheet>();` disappears as well.
//!
//! For more attributes and options see [`derive@FromDef`].

/// Generates an implementation of [`AssetPathSpec`] and [`HasResolver`] for the annotated struct or
/// enum. This enables this asset to be resolved from a file name prefix when used as a field
/// of a type implementing [`trait@FromDef`].
#[cfg(feature = "macros")]
pub use bevy_elf_macros::asset_spec;

pub use bevy_elf_core::*;
/// Implements the trait [`FromDef`]
/// for the annotated struct or enum, by converting all contained fields via their
/// [`FromDef::from_def()`]
/// implementation. So all fields have to implement [`FromDef`].
/// All primitive types, container types like [`Option`],
/// [`Vec`] and
/// [`HashMap`], as well as [`Handle`]
/// and [`AssetRef`] implement
/// [`FromDef`].
///
/// ### Type level `#[elf]` attribute
/// The def type ([`FromDef::Def`])
/// can be provided by the additional attribute `#[elf(def_type(DefType))]`.
///
/// There are the following ways to specify the def_type:
/// 1. `#[elf(def_type(Self))]` where no conversion is necessary, because the serializable type is
///    also the runtime type. `from_def()` just returns the input (`def: Self`) value as is.
/// 2. `#[elf(def_type(CustomType))]` to provide a custom serializable def type to be used.
///    That type needs to have a corresponding field with the same name for each field in `Self`
///    that should be converted and each such field must implement
///    [`FromDef`].
///    The field types must match the corresponding field's type in `Self` in terms
///    of its [`FromDef::Def`] type.
/// 3. `#[elf(def_type(()))]` - use this when the type has no fields that need serialization.
/// 4. If the additional `#[elf(def_type(...))]` attribute is omitted this macro generates a
///    def type.
///
/// ### Field/variant level `#[elf]` attribute
/// It is possible to influence resolution and def type generation by using the
/// `#[elf(...)]` attribute on the fields directly:
///
/// - `#[elf(with_spec(base_path = "base/path", extension = "ron"))]`<br>
///   overrides the [`asset_spec`] of the field's type (if it has one) used for resolution.
///   This is only relevant for types like [`Handle`] or [`AssetRef`].
///   The optional extension parameter is used for suffixing the file extension to the id, so in the ron file
///   writing just "water" is enough to reference something at `assets/tiles/water.ron`. If there
///   are multiple possible extensions for the same asset type (e.g. png and jpg)
///   omit the extension parameter in the `#[elf(with_spec(...))]`, but then you have to specify
///   the extension in the ron file, e.g. "water.png".
///
/// - `#[elf(with_spec(sub_path = "foo"))]`<br>
///   can be used as an alternative to specifying a `base_path` to make the field resolve relative
///   to the current path (the `base_path` used to resolve the containing type).
///
/// - `#[elf(with_resolver(CustomResolver))]`<br>
///   can be used to specify a custom type that implements
///   [`AssetResolver`], which is used to resolve the asset path from the string id.
///
/// - `#[elf(implicit)]`<br>
///   will omit the field in the generated def type and use the same id
///   as the parent (containing asset) to resolve the file name.
///   The `implicit` option can be combined freely with `with_spec`. Note that if the parent's
///   resolver has an extension configured, the inherited id has no extension, so the extension
///   parameter must be specified here. If the parent's resolver has no extension configured,
///   the id must include the extension explicitly, and this field's extension must match the parent's.
///   If you want implicit extensions and support multiple different extensions for the same asset
///   type you can provide a custom `AssetResolver` via `#[elf(with_resolver(CustomResolver))]`.
///
/// - `#[elf(default)]`<br>
///   will use the [`Default`] trait to construct a value, so
///   it omits the field in the generated def type and also skips resolution completely.
///
/// - `#[elf(from_default)]`<br>
///   will use the [`Default`] trait to construct the value of the
///   field's def type, so it omits the field in the generated def type and passes the default value
///   to the field type's [`from_def`](FromDef::from_def) method.
///
/// - `#[elf(on_def(#[...]))]`<br>
///   can be used on the type itself, on a variant or on a field to forward
///   the given attribute to the same place on the def type. Note that without `#[elf(on_def(#[...]))]`
///   the derive macros serde::Serialize and serde::Deserialize are hung onto the def type
///   automatically. When using `#[elf(on_def(#[...]))]` you have to specify them yourself.
///
/// - Use `#[elf(expose_resolver)]`<br>
///   on a field to generate a function on the type containing the field
///   which exposes the resolver. The name is derived from the field name (e.g.
///   `MyAsset::foo_resolver()` for the field `foo`).
///
/// # Example
/// ```ignore
/// #[derive(FromDef, Asset, TypePath)]
/// struct Spritesheet {
///     #[elf(implicit, with_spec(sub_path = "image", extension = "png"))]
///     image: Handle<Image>,
///
///     #[elf(implicit, with_spec(sub_path = "layout", extension = "tl.ron"))]
///     spritesheet_layout: Handle<TextureAtlasLayout>,
///     kind: SpritesheetKind,
/// }
///
/// #[derive(FromDef)]
/// enum SpritesheetKind {
///     Static(usize),
///     Animated(
///         #[elf(with_spec(base_path = "animations", extension = "ani.ron"))]
///         Handle<SpriteAnimationAsset>,
///     ),
/// }
/// ```
#[cfg(feature = "macros")]
pub use bevy_elf_macros::FromDef;

use bevy_app::prelude::*;

use bevy_asset::prelude::*;

/// Extension trait for [`App`] to register a ron asset with a [`FromDef`] implementation
/// and a corresponding [`RonAssetLoader`] by calling `app.init_ron_asset::<MyRonAsset>()`.
#[cfg(feature = "app")]
pub trait AppExt {
    fn init_ron_asset<A: Asset + FromDef>(&mut self) -> &mut Self;
}

#[cfg(feature = "app")]
impl AppExt for App {
    /// Registers the asset type `A` and a [`RonAssetLoader<A>`]
    /// This is equivalent to calling
    /// ```ignore
    /// app.init_asset::<A>()
    ///    .init_asset_loader::<RonAssetLoader<A>>();
    /// ```
    /// Note that the asset type `A` has to implement [`Asset`] and [`FromDef`].
    fn init_ron_asset<A: Asset + FromDef>(&mut self) -> &mut Self {
        self.init_asset::<A>()
            .init_asset_loader::<RonAssetLoader<A>>();

        self
    }
}
