# bevy_utils

Workspace for [`bevy_elf`](bevy_elf), a crate for loading and resolving [Bevy](https://bevyengine.org/)
assets that reference other assets by name.

## Crates

| Crate                                      | Description                                                          |
|--------------------------------------------|----------------------------------------------------------------------|
| [`bevy_elf`](bevy_elf)                     | The public crate. See its [README](bevy_elf/README.md) for usage.    |
| [`bevy_elf_core`](bevy_elf_core)           | Contains traits (e.g. the core trait `FromDef`), types, helper functions and `FromDef` impls for bevy  types. |
| [`bevy_elf_macros`](bevy_elf_macros)       | Implements the `FromDef` derive macro and `asset_spec` attribute. Not meant to be depended on directly — re-exported through `bevy_elf`'s `macros` feature. |
| [`elf_macro_utils`](elf_macro_utils)       | Contains helper functions and types for writing procedural macros.   |
| [`bevy_entity_lookup`](bevy_entity_lookup) | Derives mirrored structs with String EntityIds replaced by Entities and provides lookup functionality after entities are spawned |
| [`bevy_entity_lookup_macros`](bevy_entity_lookup_macros) | Contains the Lookup derive macro re-exported by bevy_entity_lookup   |
| [`bevy_event_cascade`](bevy_event_cascade)                     | Helper crate for merging different paths of event cascades (event observers triggering more events) to have observers being triggered after multiple different events where triggered |

## License

Dual-licensed under either [MIT](https://github.com/Koettlitz/bevy_utils/blob/master/LICENSE-MIT) or [Apache License, Version 2.0](https://github.com/Koettlitz/bevy_utils/blob/master/LICENSE-APACHE)
at your option.
