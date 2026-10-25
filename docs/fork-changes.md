# Fork change ledger

This ledger records every project change made in the `smacdmac101/cadcraft` fork for upstream
review. Stable change IDs identify the individual changes within a feature commit. Future
commit bodies list their IDs and validation; use `git log --oneline --grep=CC-013`, for example,
to locate the implementing commit. Entries distinguish measured results from remaining gaps.

## Baseline and first fork update

The fork started at upstream `f043540d13dac03ce895a96c78a8349f3e40bd5b`. The first published
update, [9cde0cc](https://github.com/smacdmac101/cadcraft/commit/9cde0cccd3c0ca5fd73314bbde597403b784587e),
contains CC-001 through CC-011. These records describe that existing commit without changing
its published history. Local setup established a Windows MSVC/Rust build and isolated drawing
copies for comparison; no private drawing or Autodesk asset was committed.

| ID | Problem and resulting behavior | Main files | Validation at that commit |
|---|---|---|---|
| CC-001 | Workspace dependency versions did not match the APIs used by the source and pinned craft-ui revision. Use harfrust 0.12, quick-xml 0.41 and the egui/eframe 0.36 family, with the resolved lockfile. | `Cargo.toml`, `Cargo.lock` | Windows workspace desktop build; full CI. |
| CC-002 | rstar 0.13 expects an envelope by value. Update the spatial query call. | `crates/engine/src/spatial.rs` | Engine tests and Clippy. |
| CC-003 | ATTRIB/ATTDEF field length (DXF 73) incorrectly overrode vertical justification (74). Read attributes from 74 and ordinary TEXT from 73. | `crates/io/src/dxf_read.rs`, `crates/io/tests/attribute_vertical_alignment.rs` | Regression reproduced before the fix; 180 import/save/reopen combinations passed after it. |
| CC-004 | Selecting a single attributed block did not expose its values in Properties. Add editable values and read-only constant attributes, through ATTEDIT for Undo. | `crates/ui-egui/src/palettes.rs`, `palettes/attribute_tests.rs` | Properties edit/Undo interaction test; live comparison drawing. |
| CC-005 | No attribute value dialog appeared for a double-click or selected ATTEDIT/EATTEDIT. Add Apply/OK/Cancel, effective dynamic-block names and selection/document-specific editing state. JSON commands stay dialog-free. | `attributedlg.rs`, `canvas.rs`, `dialogs.rs`, `lib.rs`, `menus.rs` in `crates/ui-egui/src` | Apply, Cancel, Undo and JSON-routing tests; live value edit/Undo. Formatting tabs remain open work. |
| CC-006 | CADCraft reversed a dimension-position flag that acadrust 0.6.5 already handles, changing placement and alternating repeated saves. Remove the obsolete inversion and require 0.6.5. | `crates/dwg/src/lib.rs`, `crates/dwg/Cargo.toml`, `crates/io/Cargo.toml` | Original raw-DWG flag and stable repeated-save tests passed unchanged. |
| CC-007 | A far-future test overflowed Windows SystemTime before reaching the calendar function. Test the calendar directly at that date, retaining its expected result. | `xtask/src/parity.rs` | Original `civil_dates` test passed on Windows. |
| CC-008 | A Unix-only test helper import caused a Windows warning. Gate that import with the test's platform condition. | `xtask/src/stats.rs` | Windows Clippy with warnings denied. |
| CC-009 | The top toolbar lacked a context menu and individual group visibility. Add checked group/side-panel controls, menu-bar recovery and saved group choices; keep background targets out of keyboard traversal. | `chrome.rs`, `menus.rs`, `panel_menu.rs`, `lib.rs` in `crates/ui-egui/src`; `apps/cadcraft/src/workspace.rs` | Button/background right-click, normal click, restore and keyboard tests; live hide/restart/restore verification. |
| CC-010 | New toolbar controls needed localization. Add Ukrainian reference-catalog entries. | `crates/ui-egui/src/i18n/uk.tsv` | Translation coverage and catalog validation. |
| CC-011 | Parity notes did not cover this Windows comparison. Record attribute behavior, local visual/round-trip evidence and the remaining formatting/ribbon gaps. | `docs/ui-parity.md`, `docs/file-format-parity.md` | Original drawing preserved; reported crop checked against AutoCAD 2027 and after CADCraft DXF/DWG reopening. One xref was unavailable in the isolated copy; this does not establish lossless whole-file preservation. |

**Validation for 9cde0cc:** 2,261 tests passed; all six `cargo xtask ci` steps passed (fmt,
Clippy, tests, assets, layering, wasm). `cargo build --workspace --bins --locked -j 4` passed.
The wasm check emitted three dead-code warnings in existing canvas rebuild code.

## Tabbed ribbon and ongoing records

Implementing commit: the ribbon feature commit establishes CC-012 and lists CC-013 through
CC-020 in its body. Locate an entry with `git log --oneline --grep=CC-013`.

| ID | Trigger and resulting behavior | Main files | Validation |
|---|---|---|---|
| CC-012 | The owner requested records for every previous and future change. Introduce this ledger and require change IDs, evidence and parity updates in future contribution work. | `AGENTS.md`, `docs/fork-changes.md` | Cross-check CC-001–CC-011 against the published initial fork diff. |
| CC-013 | A flat toolbar could not organize drafting tools by task. Add Home, Insert, Annotate, Parametric, View, Manage and Output tabs with named command panels, primary tools, secondary tools and overflow actions. | `crates/ui-egui/src/ribbon.rs`, `lib.rs`, `menus.rs` | Command-registry coverage and real tab/tool interaction tests; final build results recorded below. |
| CC-014 | Ribbon customization must survive closing the app and remain recoverable. Add per-tab panel visibility, tab visibility, titles, minimization, Restore Ribbon and RIBBON/RIBBONCLOSE commands. Save preferences with the native workspace. | `ribbon.rs`, `panel_menu.rs`, `apps/cadcraft/src/workspace.rs` | Hide/restore, tab-specific choices, malformed-input atomicity, old-settings defaults and disk save/reload tests. |
| CC-015 | A second full tool row duplicates the ribbon. Use compact quick access while the ribbon is shown; preserve the full classic toolbar when it is closed. Keep classic toolbar regressions explicit about their mode. | `chrome.rs`, `panel_menu.rs`; `tests/chrome_geometry.rs`, `tests/measure_distance_snapping.rs` | Classic overflow, keyboard traversal, snapping and ribbon recovery checks. |
| CC-016 | New tabs, controls and tools need translation coverage. Add reference translations and include the ribbon catalog in localization checks. | `crates/ui-egui/src/i18n/{tests.rs,uk.tsv}` | Catalog, marked-literal and command/menu/tool coverage tests. |
| CC-017 | Upstream reviewers need reproducible behavior and an honest parity scope. Document commands, acceptance cases, limitations and validation for the ribbon. | `docs/ui-parity.md`, `docs/control-protocol.md`, this ledger; `crates/ui-egui/tests/ribbon.rs` | Full CI, Windows build and visual verification passed; detailed results below. |
| CC-018 | Floating palettes had Float/Move-to-group controls without a clear return to a side. Add Dock Left/Right to title menus and floating headers, preserving bounded palette width. Make close idempotent and keep the selected tool presentation when resetting panel layout. | `crates/ui-egui/src/docking.rs`, `tests/ribbon.rs` | Floating-to-left/right interaction, width retention and invalid-zone atomicity checks; live floating-header and title-menu verification passed. |
| CC-019 | Tool Sets duplicated the ribbon. Add exclusive Move to Ribbon/Move to Tool Sets actions that retain palette placement and command visibility. Reuse both Drafting and Modeling catalogs, include Hatch in Draw overflow, keep unsupported Modeling tools disabled, and save shared hidden-tool settings. Keep the classic presentation as the compatible default until moved. | `ribbon.rs`, `lib.rs`, `menus.rs`, `apps/cadcraft/src/workspace.rs`; catalog and interaction tests | Catalog completeness, presentation round-trip, Modeling placeholders, hidden-tool/placement preservation and native workspace save/reload tests. |
| CC-020 | Palette behavior differed and lacked hover auto-hide. All side-panel title menus share Dock Left/Right, Float and checked Auto-hide; only Tool Sets offers Move to Ribbon, now located in its right-click menu. Docked palettes collapse to edge tabs, floating palettes to title bars; hover opens an overlay and leaving closes it after a short delay. Keep Open pins the saved placement. Floating bars own their input layer so the canvas cannot consume hover, and closing schedules the removal frame even when the mouse stops. Persist preferences separately from closed visibility and translate resize gestures back to the saved layout. | `crates/ui-egui/src/docking.rs`, `docking/auto_hide.rs`, `i18n/{tests.rs,uk.tsv}`; `tests/palette_auto_hide.rs`, `tests/ribbon.rs`; native workspace test and protocol/parity docs | All five palette types at left/right/floating positions, title-menu availability, hover/leave/pin, idle repaint scheduling, persistence, malformed-input/protected-canvas checks and resize-path regression; final results below. |

**Validation for CC-012–CC-020 (Windows, 2026-10-24):** 2,280 tests passed, none failed or
ignored; all six `cargo xtask ci` steps passed (fmt, Clippy with warnings denied, tests,
assets, layering and wasm). The wasm check retains three existing dead-code warnings in
`canvas/rebuild.rs`. `cargo build --workspace --bins --locked -j 4` passed.

The focused `cargo test --workspace --test ribbon --test palette_auto_hide --locked -j 4`
run passed all 16 interaction tests. Native checks covered ribbon tabs and panel geometry,
the Tool Sets title-menu move, floating-header/title-menu redocking, hover opening and
pinning for docked/floating palettes, readable edge labels, saved preferences after restart,
and automatic closing when the pointer leaves and stops. Opening/closing a hover panel
left the drawing view unchanged. Tests also retain the drawing identity and dirty state.
Original drawings were preserved; the final visual pass used CADCraft's sample drawing.

The new ribbon reuses CADCraft's original icons and existing command implementations. Floating ribbon
panels, user-authored ribbon layouts, contextual object-specific tabs and AutoCAD's temporary
minimized flyouts remain outside this change. The optional Modeling tab exposes the existing
catalog; it does not add a 3D modeling kernel. Public behavior references:
[AutoCAD 2027 ribbon](https://help.autodesk.com/cloudhelp/2027/ENU/AutoCAD-LT/files/GUID-D20EF1D7-4135-48A7-B68E-65BF3BFF3D70.htm).
[Palette docking](https://help.autodesk.com/view/ACD/2027/ENU/?caas=caas%2Fdocumentation%2FACD%2F2014%2FENU%2Ffiles%2FGUID-893A3C0A-C735-49E4-878E-C8641DEAD618-htm.html).
[Palette auto-hide/display properties](https://help.autodesk.com/view/ACD/2027/ENU/?caas=caas%2Fdocumentation%2FACD%2F2014%2FENU%2Ffiles%2FGUID-FCA90242-7748-482B-BA16-E70C7511F468-htm.html).

## Ribbon display-state comparison follow-up

Reference: AutoCAD 2027's ribbon display menu was observed on Windows without editing the
open drawing or changing its workspace preferences. Public
[display-state documentation](https://help.autodesk.com/cloudhelp/2027/ENU/AutoCAD-LT/files/GUID-627FCA86-38B1-404B-BB09-9FC3348B54F2.htm)
supports the selected minimize modes and cycle order. CADCraft retains its own icons and code.

| ID | Trigger and resulting behavior | Main files | Validation |
|---|---|---|---|
| CC-021 | Only tabs-only minimization existed. Add panel-title and panel-button presentations, the full display cycle and a dedicated display menu. Keep the prior minimized boolean and tabs-only defaults compatible; persist the preferred mode and cycle position. Include reference translations. | `ribbon.rs`, `menus.rs`, `i18n/uk.tsv`, native workspace save/reload test | Mode geometry, cycle order, hidden-panel retention, legacy defaults and native settings round-trip. |
| CC-022 | Collapsed tools required permanently expanding the ribbon. Add temporary tab and panel flyouts using the same tools, visibility and command routing; dismiss on Escape, outside clicks or tool invocation. Full-tab flyouts use the available window width instead of the generic popup width; single-panel flyouts stay compact. Compact panel buttons and chevrons use CADCraft's existing drawn icons, avoiding missing-font triangle glyphs. | `ribbon.rs`, `tests/ribbon.rs`; protocol and parity docs | Tool dispatch without persistent expansion, single-panel contents and dismissal tests; native visual check. |
| CC-023 | Auto-hide used a different body identity from normal docking, risking lost editor focus/selection and scroll state on pinning. Use the docking renderer's stable panel body identity in hover views. | `docking/auto_hide.rs`, `tests/palette_auto_hide.rs` | QuickCalc editor identity retained across floating, auto-hide and pinning. |
| CC-024 | An explicitly invalid tab in panel configuration silently fell back to the active tab. Reject invalid or incorrectly typed tab IDs and unknown display modes atomically. | `ribbon.rs`, `tests/ribbon.rs` | Preferences remain unchanged after malformed requests; out-of-range cycle state rejected. |

Validation on Windows, 2026-10-24: all six `cargo xtask ci` steps passed (fmt, Clippy,
tests, assets, layering, wasm); 2,288 tests passed with no failures or ignored tests.
The focused ribbon/palette suites passed all 24 interaction tests, and
`cargo build --workspace --bins --locked -j 4` passed. Native checks confirmed compact
panel icons/chevrons, single-panel and full-width tab flyouts, narrow-window scrolling,
tool dispatch without an accidental canvas point or drawing modification, unchanged
canvas geometry while flyouts open, and display settings retained after restart.
The wasm check emitted three existing dead-code warnings in canvas rebuild code.
Remaining gaps include floating ribbon panels, authored layouts, contextual object tabs and
native OS palette windows. This follow-up does not add unsupported modeling commands.
