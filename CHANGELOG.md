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

<!-- contributors -->
[sudo-carson]: https://github.com/sudo-carson

<!-- Pull requests -->
[#3]: https://github.com/AlvaroMS25/zephyrus/pull/3
