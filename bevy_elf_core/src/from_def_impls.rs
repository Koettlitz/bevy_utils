macro_rules! from_def_self {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::FromDef for $ty {
                type Def = Self;

                fn from_def(
                    def: Self::Def,
                    _: &mut bevy_asset::LoadContext<'_>,
                ) -> Result<Self, crate::FromDefError> {
                    Ok(def)
                }
            }
        )+
    };
}

from_def_self![
    (),
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize,
    f32,
    f64,
    bool,
    String,
    std::time::Duration,
];

/// Generates [`FromDef`](crate::FromDef) implementations for a set of types, gated
/// behind a cargo feature.
///
/// Expands to a module named `$mod_name`, annotated with `#[cfg(feature = $feature)]`,
/// that `use`s the given `types` from `$krate` and implements [`FromDef`](crate::FromDef)
/// for each of them. Each implementation is a no-op passthrough: `Def` is `Self`, and
/// `from_def` simply returns the value unchanged.
///
/// This is intended for third-party types that are already in their final, usable form
/// and don't need any resolution or conversion step — they just need to satisfy the
/// [`FromDef`](crate::FromDef) bound so they can be used as fields in other `Def` types.
///
/// # Example
///
/// ```ignore
/// from_def_types!(
///     mod math,
///     feature = "math",
///     use bevy_math,
///     types = [Vec2, Vec3, Quat],
/// );
/// ```
macro_rules! from_def_types {
    (
        mod $mod_name:ident,
        feature = $feature:literal,
        use $krate:path,
        types = [$($ty:ident),+ $(,)?]
    ) => {
        #[cfg(feature = $feature)]
        mod $mod_name {
            use $krate::{$($ty),+};

            from_def_self![$($ty),+];
        }
    };
}

from_def_types!(
    mod math,
    feature = "math",
    use bevy_math,
    types = [
        Affine2, Affine3, Affine3A, BVec2, BVec3, BVec4,
        DAffine2, DAffine3, DMat2, DMat3, DMat4, DQuat, DVec2, DVec3, DVec4,
        Dir2, Dir3, Dir4, Dir3A,
        IRect, IVec2, IVec3, IVec4, Isometry2d, Isometry3d, Mat2, Mat3, Mat4, Mat3A,
        Quat, Rect, Rot2,
        URect, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4, Vec3A ]
);

from_def_types!(
    mod image,
    feature = "image",
    use bevy_image,
    types = [TextureAtlasLayout]
);
