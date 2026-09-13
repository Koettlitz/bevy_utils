use std::{collections::HashMap, hash::Hash};

use bevy_ecs::entity::Entity;

use crate::{Lookup, MissingEntity};

impl Lookup for String {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(self.clone())
    }
}

impl Lookup for usize {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for u8 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for u16 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for u32 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for u64 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for isize {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for i8 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for i16 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for i32 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for i64 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for f32 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for f64 {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for bool {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(*self)
    }
}

impl Lookup for () {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(())
    }
}

#[cfg(feature = "bevy_asset")]
impl<T: bevy_asset::Asset> Lookup for bevy_asset::Handle<T> {
    type LookedUp = Self;

    fn lookup(&self, _: &crate::LookupMap) -> std::result::Result<Self::LookedUp, MissingEntity> {
        Ok(self.clone())
    }
}

impl Lookup for crate::EntityRef {
    type LookedUp = Entity;

    fn lookup(
        &self,
        lookup: &crate::LookupMap,
    ) -> std::result::Result<Self::LookedUp, MissingEntity> {
        let id = self.id();
        lookup.lookup(&id).copied().ok_or(MissingEntity(id))
    }
}

impl<T: Lookup> Lookup for Option<T> {
    type LookedUp = Option<T::LookedUp>;

    fn lookup(
        &self,
        lookup: &crate::LookupMap,
    ) -> std::result::Result<Self::LookedUp, crate::MissingEntity> {
        self.as_ref().map(|t| t.lookup(lookup)).transpose()
    }
}

impl<T: Lookup> Lookup for Vec<T> {
    type LookedUp = Vec<T::LookedUp>;

    fn lookup(
        &self,
        lookup: &crate::LookupMap,
    ) -> std::result::Result<Self::LookedUp, crate::MissingEntity> {
        self.iter().map(|t| t.lookup(lookup)).collect()
    }
}

impl<K, T> Lookup for HashMap<K, T>
where
    K: Hash + Eq + Clone,
    T: Lookup,
{
    type LookedUp = HashMap<K, T::LookedUp>;

    fn lookup(
        &self,
        lookup: &crate::LookupMap,
    ) -> std::result::Result<Self::LookedUp, crate::MissingEntity> {
        self.iter()
            .map(|(k, v)| Ok((k.clone(), v.lookup(lookup)?)))
            .collect()
    }
}
