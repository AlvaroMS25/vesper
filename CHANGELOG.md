# Changelog

Log of changes for ``zephyrus`` crate, changes in between versions will be documented here.

---

## 0.5.0 - 2022-11-23

#### Changes:
- Updated twilight to version 0.14 ([sudo-carson] at [#3])


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
- Added `Framework#twilight_commands` to get a serializable representation of registered commands ([carterhimmel] at [#9] & [#10])

## 0.11.0 -- 2023/09/21
- Use typed errors
- Added localizations for command both commands and it's attributes
- Allow a `#[skip]` attribute for chat command arguments

## 0.12.0 -- 2023/12/28
- Fixed an error when dereferencing a misaligned pointer on `Range` type ([iciivy] at [#13])
- Allow providing localizations using closures and function pointers

<!-- contributors -->
[sudo-carson]: https://github.com/sudo-carson
[carterhimmel]: https://github.com/carterhimmel
[iciivy]: https://github.com/iciivy

<!-- Pull requests -->
[#3]: https://github.com/AlvaroMS25/zephyrus/pull/3
[#9]: https://github.com/AlvaroMS25/zephyrus/pull/9
[#10]: https://github.com/AlvaroMS25/zephyrus/pull/10
[#13]: https://github.com/AlvaroMS25/vesper/pull/13
