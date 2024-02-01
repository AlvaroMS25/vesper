# Changelog

Log of changes for ``zephyrus`` crate, changes in between versions will be documented here.

---

## 0.5.0 - 2022-11-23

#### Changes:
- Updated twilight to version 0.14 ([Carson M] at [#3])


## 0.6.0 -- 2022-12-28

#### Changes:
- Framework can now have custom return types from commands
- Created usage examples

## 0.7.0 -- 2023-01-05

#### Changes:
- Changed Parse trait signature
- Implemented parse for Id<AttachmentMarker>
- Now a mutable reference for an interaction can be obtained using SlashContext::interaction_mut()
- Implemented Parse for Attachment, User and Role

## 0.8.0 -- 2023-02-06

#### Changes:
- Created Modal trait
- Now context can send modals directly
- Created Modal derive macro to create modals directly

## 0.9.0 -- 2023-02-24

#### Changes:
- Updated twilight dependencies to 0.15
- Now modal derive requires to specify attributes inside a modal one: #[modal(...)]
- Now parse derive requires to specify attributes inside a parse one: #[parse(...)]

## 0.10.0 -- 2023-07-31

#### Changes:
- Moved to `darling` crate to create macros
- Added support for `chat` and `message` commands
- Added `#[only_guilds]` and `#[nsfw]` attribute for commands
- Deprecated `SlashContext#acknowledge` in favor of `SlashContext#defer`
- Added `Framework#twilight_commands` to get a serializable representation of registered commands ([Carter] at [#9] & [#10])

## 0.11.0 -- 2023/09/21
- Use typed errors
- Added localizations for command both commands and it's attributes
- Allow a `#[skip]` attribute for chat command arguments

## 0.12.0 -- 2023/12/28
- Fixed an error when dereferencing a misaligned pointer on `Range` type ([iciivy] at [#13])
- Allow providing localizations using closures and function pointers

## 0.13.0 -- 2024/x/x
- Omit emitting argument parsing code on commands without arguments ([Carson M] at [#16])
- Disabled ``twilight-http`` default features ([Carson M] at [#17])

<!-- contributors -->
[Carson M]: https://github.com/decahedron1
[Carter]: https://github.com/Fyko
[iciivy]: https://github.com/iciivy

<!-- Pull requests -->
[#3]: https://github.com/AlvaroMS25/zephyrus/pull/3
[#9]: https://github.com/AlvaroMS25/zephyrus/pull/9
[#10]: https://github.com/AlvaroMS25/zephyrus/pull/10
[#13]: https://github.com/AlvaroMS25/vesper/pull/13
[#16]: https://github.com/AlvaroMS25/vesper/pull/16
[#17]: https://github.com/AlvaroMS25/vesper/pull/17
