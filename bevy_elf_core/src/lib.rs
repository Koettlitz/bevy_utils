use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::marker::PhantomData;

use bevy_asset::io::Reader;
use bevy_asset::prelude::*;
use bevy_asset::{AssetLoader, AssetPath, LoadContext, ParseAssetPathError};
use bevy_reflect::TypePath;
use ron::de::SpannedError;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

mod from_def_impls;

type Phantom<L> = PhantomData<fn() -> L>;

/// Resolves the [`AssetPath`] for a given string id.
pub trait AssetResolver {
    fn resolve(&self, asset_id: &str) -> Result<AssetPath<'static>, ResolveError>;
}

/// Resolves the [`AssetPath`] for a given string asset id from a static context, meaning to self
/// object is nessecary.
pub trait StaticAssetResolver {
    fn resolve(asset_id: &str) -> Result<AssetPath<'static>, ResolveError>;
}

/// An asset resolver, that assumes the given string id is the complete asset path and returns it
/// as an [`AssetPath`](`bevy_asset::AssetPath`) unchanged.
pub struct PathResolver;
impl AssetResolver for PathResolver {
    fn resolve(&self, asset_path: &str) -> Result<AssetPath<'static>, ResolveError> {
        Ok(AssetPath::from(asset_path.to_string()))
    }
}

/// Extracts the string id of the asset from its [`AssetPath`].
pub fn extract_id_from(asset_path: AssetPath) -> Result<String, ResolveError> {
    asset_path
        .path()
        .file_prefix()
        .ok_or_else(|| ResolveError::MissingFileName(asset_path.clone_owned()))
        .map(|v| v.to_string_lossy().to_string())
}

#[derive(Error, Debug)]
pub enum FromDefError {
    #[error("{0}")]
    Resolve(#[from] ResolveError),

    #[error("{0}")]
    InvalidDef(String),
}

impl From<ParseAssetPathError> for FromDefError {
    fn from(value: ParseAssetPathError) -> Self {
        Self::from(ResolveError::from(value))
    }
}

#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("{0}")]
    Parse(#[from] ParseAssetPathError),

    #[error("missing file name in asset path {0}")]
    MissingFileName(AssetPath<'static>),

    #[error("invalid asset link \"{0}\"")]
    InvalidAssetLink(String),
}

/// An adapter type, that implements [`AssetResolver`] by delegating to a
/// [`StaticAssetResolver`] (`S`)
pub struct StaticResolverAdapter<S>(Phantom<S>);

impl<S> Default for StaticResolverAdapter<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S: StaticAssetResolver> AssetResolver for StaticResolverAdapter<S> {
    fn resolve(&self, asset_id: &str) -> Result<AssetPath<'static>, ResolveError> {
        S::resolve(asset_id)
    }
}

pub trait HasResolver {
    type Resolver: AssetResolver;

    fn resolver() -> Self::Resolver;
}

impl<T: AssetResolver + Default> HasResolver for T {
    type Resolver = Self;

    fn resolver() -> Self::Resolver {
        Self::default()
    }
}

pub trait AssetPathSpec {
    const BASE_PATH: &'static str;
    const RESOLVE_STRATEGY: ResolveStrategy;
}

#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub enum ResolveStrategy {
    #[default]
    FileNameIsId,
    FileStemIsId {
        extension: &'static str,
    },
    DirectoryNameIsId {
        file_name: &'static str,
    },
}

/// An adapter type, that implements [`AssetResolver`] by using an [`AssetPathSpec`] (`S`).
pub struct ResolverSpec<S>(Phantom<S>);

impl<S> Default for ResolverSpec<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S: AssetPathSpec> AssetPathSpecProvider for ResolverSpec<S> {
    fn base_path(&self) -> Cow<'static, str> {
        Cow::Borrowed(S::BASE_PATH)
    }

    fn resolve_strategy(&self) -> ResolveStrategy {
        S::RESOLVE_STRATEGY
    }
}

pub struct DynamicPathResolver {
    pub base_path: String,
    pub resolve_strategy: ResolveStrategy,
}

impl DynamicPathResolver {
    pub fn resolve_sub_path(
        base_path: &str,
        sub_path: &str,
        resolve_strategy: ResolveStrategy,
    ) -> Self {
        let base_path = AssetPath::parse(base_path);
        Self {
            base_path: base_path.resolve(&AssetPath::parse(sub_path)).to_string(),
            resolve_strategy,
        }
    }

    pub fn resolve_sub_path_from_load_context(
        load_context: &LoadContext,
        sub_path: &str,
        resolve_strategy: ResolveStrategy,
    ) -> Result<Self, ParseAssetPathError> {
        let sub_path = AssetPath::parse(sub_path);
        let base_path = load_context
            .path()
            .parent()
            .map(|p| p.resolve(&sub_path))
            .unwrap_or_else(|| sub_path)
            .to_string();

        Ok(Self {
            base_path,
            resolve_strategy,
        })
    }
}

impl AssetPathSpecProvider for DynamicPathResolver {
    fn base_path(&self) -> Cow<'static, str> {
        Cow::Owned(self.base_path.clone())
    }

    fn resolve_strategy(&self) -> ResolveStrategy {
        self.resolve_strategy
    }
}

pub struct SpecResolver<S>(Phantom<S>);

impl<S> Default for SpecResolver<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S: HasSpecProvider> AssetPathSpecProvider for SpecResolver<S> {
    fn base_path(&self) -> Cow<'static, str> {
        S::provider().base_path()
    }

    fn resolve_strategy(&self) -> ResolveStrategy {
        S::provider().resolve_strategy()
    }
}

/// An [`AssetResolver`] using a `base_path` and an optional file `extension`.
pub trait AssetPathSpecProvider {
    fn base_path(&self) -> Cow<'static, str>;
    fn resolve_strategy(&self) -> ResolveStrategy;
}

pub trait HasSpecProvider {
    type Provider: AssetPathSpecProvider;

    fn provider() -> Self::Provider;
}

impl<T> HasSpecProvider for T
where
    T: AssetPathSpecProvider + Default,
{
    type Provider = Self;

    fn provider() -> Self::Provider {
        Self::default()
    }
}

impl<T> AssetResolver for T
where
    T: AssetPathSpecProvider,
{
    fn resolve(&self, asset_id: &str) -> Result<AssetPath<'static>, ResolveError> {
        let base_path = AssetPath::from(self.base_path().to_string());

        Ok(match self.resolve_strategy() {
            ResolveStrategy::FileNameIsId => {
                base_path.resolve(&AssetPath::from(asset_id.to_owned()))
            }
            ResolveStrategy::FileStemIsId { extension } => {
                let file_name = format!("{asset_id}.{extension}");
                base_path.resolve(&AssetPath::from(file_name))
            }
            ResolveStrategy::DirectoryNameIsId { file_name } => base_path
                .resolve(&AssetPath::from(asset_id.to_owned()))
                .resolve(&AssetPath::from(file_name)),
        })
    }
}

/// Represents a type, that can be constructed from a deserializable def type.
/// An asset type implementing [`trait@FromDef`] can be loaded by the [`RonAssetLoader`].
/// It deserializes the asset bytes into the [`FromDef::Def`] type and then turns it into
/// the runtime asset type which implements [`trait@FromDef`] by passing it to [`FromDef::from_def()`].
/// This trait can be implemented manually or by using the [`derive@FromDef`] derive macro.
/// To enable loading ron assets implementing [`trait@FromDef`] init the asset and loader via
/// [`AppExt::init_ron_asset()`].
pub trait FromDef {
    type Def: DeserializeOwned;

    fn from_def(def: Self::Def, ctx: &mut LoadContext) -> Result<Self, FromDefError>
    where
        Self: Sized;
}

/// Like [`trait@FromDef`], but uses an explicitly passed [`AssetResolver`] to resolve its [`AssetPath`].
pub trait FromDefWithResolver {
    type Def: DeserializeOwned;
    type Error;

    fn from_def_with_resolver<R: AssetResolver>(
        def: Self::Def,
        resolver: &R,
        ctx: &mut LoadContext,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// A [`Handle`] with the assets string id preserved.
#[derive(Debug, Eq)]
pub struct AssetRef<A: Asset> {
    id: String,
    handle: Handle<A>,
}

impl<A: Asset> AssetRef<A> {
    pub fn new(id: String, handle: Handle<A>) -> Self {
        Self { id, handle }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn handle(&self) -> &Handle<A> {
        &self.handle
    }
}

impl<A: Asset + HasResolver> FromDef for AssetRef<A> {
    type Def = String;

    fn from_def(def: Self::Def, ctx: &mut LoadContext) -> Result<Self, FromDefError>
    where
        Self: Sized,
    {
        let handle = ctx.load(A::resolver().resolve(&def)?);
        Ok(Self { id: def, handle })
    }
}

impl<A: Asset> FromDefWithResolver for AssetRef<A> {
    type Def = String;
    type Error = ResolveError;

    fn from_def_with_resolver<R: AssetResolver>(
        def: Self::Def,
        resolver: &R,
        ctx: &mut LoadContext,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            handle: ctx.load(resolver.resolve(&def)?),
            id: def,
        })
    }
}

impl<A: Asset> PartialEq for AssetRef<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.handle.id() == other.handle.id()
    }
}

impl<A: Asset> PartialEq<Handle<A>> for AssetRef<A> {
    fn eq(&self, other: &Handle<A>) -> bool {
        self.handle.id() == other.id()
    }
}

impl<A: Asset> Clone for AssetRef<A> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            handle: self.handle.clone(),
        }
    }
}
impl<A: Asset + HasResolver> FromDef for Handle<A> {
    type Def = String;

    fn from_def(def: Self::Def, ctx: &mut LoadContext) -> Result<Self, FromDefError> {
        Ok(ctx.load(A::resolver().resolve(&def)?))
    }
}

impl<A: Asset> FromDefWithResolver for Handle<A> {
    type Def = String;
    type Error = FromDefError;

    fn from_def_with_resolver<R: AssetResolver>(
        def: Self::Def,
        resolver: &R,
        ctx: &mut LoadContext,
    ) -> Result<Self, Self::Error> {
        Ok(ctx.load(resolver.resolve(&def)?))
    }
}

impl<T, D> FromDef for Option<T>
where
    T: FromDef<Def = D>,
    D: DeserializeOwned,
{
    type Def = Option<D>;

    fn from_def(def: Self::Def, ctx: &mut LoadContext) -> Result<Self, FromDefError> {
        def.map(|d| T::from_def(d, ctx)).transpose()
    }
}

impl<T, D> FromDefWithResolver for Option<T>
where
    T: FromDefWithResolver<Def = D>,
    D: DeserializeOwned,
{
    type Def = Option<D>;
    type Error = T::Error;

    fn from_def_with_resolver<R: AssetResolver>(
        def: Self::Def,
        resolver: &R,
        ctx: &mut LoadContext,
    ) -> Result<Self, Self::Error> {
        def.map(|d| T::from_def_with_resolver(d, resolver, ctx))
            .transpose()
    }
}

impl<T, D> FromDef for Vec<T>
where
    T: FromDef<Def = D>,
    D: DeserializeOwned,
{
    type Def = Vec<D>;

    fn from_def(def: Self::Def, ctx: &mut LoadContext) -> Result<Self, FromDefError> {
        def.into_iter().map(|d| T::from_def(d, ctx)).collect()
    }
}

impl<T, D> FromDefWithResolver for Vec<T>
where
    T: FromDefWithResolver<Def = D>,
    D: DeserializeOwned,
{
    type Def = Vec<D>;
    type Error = T::Error;

    fn from_def_with_resolver<R: AssetResolver>(
        def: Self::Def,
        resolver: &R,
        ctx: &mut LoadContext,
    ) -> Result<Self, Self::Error> {
        def.into_iter()
            .map(|d| T::from_def_with_resolver(d, resolver, ctx))
            .collect()
    }
}

impl<A, K, D> FromDef for HashMap<K, A>
where
    A: FromDef<Def = D>,
    K: DeserializeOwned + Eq + Hash,
    D: DeserializeOwned,
{
    type Def = HashMap<K, D>;

    fn from_def(def: Self::Def, ctx: &mut LoadContext) -> Result<Self, FromDefError> {
        def.into_iter()
            .map(|(k, d)| Ok((k, A::from_def(d, ctx)?)))
            .collect()
    }
}

impl<A, K, D> FromDefWithResolver for HashMap<K, A>
where
    A: FromDefWithResolver<Def = D>,
    K: DeserializeOwned + Eq + Hash,
    D: DeserializeOwned,
{
    type Def = HashMap<K, D>;
    type Error = A::Error;

    fn from_def_with_resolver<R: AssetResolver>(
        def: Self::Def,
        resolver: &R,
        ctx: &mut LoadContext,
    ) -> Result<Self, Self::Error> {
        def.into_iter()
            .map(|(k, d)| Ok((k, A::from_def_with_resolver(d, resolver, ctx)?)))
            .collect()
    }
}

/// Loads assets, which implement [`trait@FromDef`] from ron files passing the deserialized
/// [`FromDef::Def`] value into the assets [`FromDef::from_def`] method.
#[derive(TypePath)]
pub struct RonAssetLoader<A>(Phantom<A>);
impl<A> AssetLoader for RonAssetLoader<A>
where
    A: FromDef + Asset,
{
    type Asset = A;
    type Error = RonAssetLoadError;
    type Settings = ();

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let def: A::Def = ron::de::from_bytes(&bytes)?;
        Ok(A::from_def(def, load_context)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

impl<A> Default for RonAssetLoader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Error, Debug)]
pub enum RonAssetLoadError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Spanned(#[from] SpannedError),
    #[error("{0}")]
    FromDef(#[from] FromDefError),
}
