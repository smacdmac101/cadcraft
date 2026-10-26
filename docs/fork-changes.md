# Fork change ledger

This ledger records every project change made in the `smacdmac101/cadcraft` fork for upstream
review. Stable change IDs identify the individual changes within a feature commit. Future
commit bodies list their IDs and validation; use `git log --oneline --grep=CC-013`, for example,
to locate the implementing commit. Entries distinguish measured results from remaining gaps.

## Reversal and dependency tracking

From CC-039 onward, each independently useful feature arc is a separate commit containing
its implementation, tests, translations and records. Each entry states its dependencies,
affected behavior and reversal scope. Revert that commit as a unit, then run the quality
gates; retain the original commit and add a reversal record rather than rewriting history.
Before reverting, inspect later entries for dependents. A dependent must be reverted first
or adapted and verified, and the interaction must be recorded explicitly. Sharing a file
does not by itself mean sharing behavior; resolve textual conflicts without discarding
unrelated later work.

Earlier published history contains several IDs per commit. Those IDs are traceable but
are **not independent Git revert units**. In particular, `9cde0cc` groups CC-001–011;
`029d948` groups the ribbon/palette/record work CC-012–020; subsequent display/context/native/
anchor arcs build on that ribbon foundation. `fc0c091` groups CC-035–038: reverting it
removes the inventory, Layout context, status customization and popup Escape protection
together. Reversing one historical ID requires a focused inverse patch with its own tests;
do not revert its whole commit and claim the other IDs are unaffected. The compatibility
baseline (CC-001/002/006–008) also supports later Windows builds. Published hashes are kept.

| Published revert unit | IDs | Dependencies / effect on other arcs |
|---|---|---|
| `9cde0cc` | CC-001–011 | Windows compatibility baseline plus attribute/toolbar work. Later builds rely on the compatibility fixes; use an inverse patch for one behavior. |
| `029d948` | CC-012–020 | Ribbon foundation and shared palette controls. Later ribbon features require this foundation. |
| `fbc8ee1` | CC-021–024 | Display modes/flyouts, palette identity and validation. Depends on `029d948`; later ribbon rendering uses these modes. |
| `f8dad0f` | CC-025–027 | Detached panels/contextual tabs. Depends on ribbon/display modes; native panel windows and Layout context build on it. |
| `b3f794f` | CC-028–031 | Desktop panels/geometry/diagnostics and whole ribbon. Depends on detached panels; anchoring/edge docking uses this native-window foundation. |
| `27403b3` | CC-032–034 | Whole-ribbon anchors, roll-up and edge docking. Depends on `b3f794f`; reverting loses those three placement behaviors together. |
| `fc0c091` | CC-035–038 | Inventory/Layout/status/Escape. Layout depends on contextual tabs; new popup controls rely on Escape protection. Whole-commit reversal removes all four IDs. |

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

## Floating panels and contextual tabs (2026-10-25)

Reference: AutoCAD 2027's public [ribbon behavior](https://help.autodesk.com/cloudhelp/2027/ENU/AutoCAD-LT/files/GUID-D20EF1D7-4135-48A7-B68E-65BF3BFF3D70.htm)
describes detached panels retained across tab switches and contextual tabs selected by object
or command state. The installed Windows ribbon was inspected; detachment semantics follow the public
reference. Context references include the [Hatch Editor](https://help.autodesk.com/view/ACD/2027/ENU/?caas=caas%2Fdocumentation%2FACD%2F2014%2FENU%2Ffiles%2FGUID-B3FF6503-8FFB-4F1A-A603-52A6408F0D66-htm.html)
and [Table](https://help.autodesk.com/view/ACD/2027/ENU/?caas=caas%2Fdocumentation%2FCIV3D%2F2014%2FENU%2FfilesACD%2FGUID-367470A6-6E6E-4181-9E53-9B0EC88F50DC-htm.html).
CADCraft uses its own existing icons and commands. Its Table context activates on a whole-table
selection; it does not claim AutoCAD's cell-selection editor or formatting ribbon.

| ID | Trigger and resulting behavior | Main files | Validation |
|---|---|---|---|
| CC-025 | Tools could only appear in the ribbon or temporary flyouts. Add Float Panel to panel context menus, title-drag detachment, compact movable persistent panels and explicit Return to Ribbon/Return All recovery. Panels retain their tab-qualified identity, remain across tab switches and return to the original source slot; close also returns tools. | `crates/ui-egui/src/ribbon.rs`, `ribbon/floating.rs`, `menus.rs`; ribbon interaction tests | Drag, command dispatch, return placement and canvas input isolation; final results below. |
| CC-026 | Object/editor selection did not expose related ribbon tools. Add accented Hatch Editor, Table and Block Editor tabs from homogeneous selections/BEDIT, with editor priority, document transitions, manual regular-tab choice, preserved context during visibility customization, readable selected labels, automatic fallback and an opt-out. Keep runtime context separate from saved regular-tab preferences; contextual detached panels only show in matching contexts. | `ribbon/context.rs`, `ribbon.rs`, `lib.rs`; ribbon interaction tests | Selection, mixed types, editor entry/exit, document switching, manual choice and disabled-context checks. |
| CC-027 | New panel state must be safe, compatible and reviewable. Persist positions and contextual preferences with legacy defaults; reject malformed, non-finite, oversized or duplicate panel state atomically; constrain floating controls to the application window and recover through reset/return. Extend Ukrainian reference coverage and control/parity documentation. | `apps/cadcraft/src/workspace.rs`, `ribbon.rs`, `ribbon/floating.rs`, `i18n/{tests.rs,uk.tsv}`, `docs/{control-protocol.md,ui-parity.md,fork-changes.md}` | Native workspace round-trip, invalid-input/uniqueness/recovery tests, translation checks and full CI. |

Validation on Windows, 2026-10-25: all six `cargo xtask ci` steps passed on the final
source (fmt, Clippy, tests, assets, layering, wasm); 2,297 tests passed, with no failures
or ignored tests. The ribbon/palette interaction suites passed all 33 cases, including
title/header dragging, panel return placement, context lifetime, mixed selection,
document switching, visibility customization and canvas input isolation.
`cargo build --workspace --bins --locked -j 4` passed. Native checks confirmed compact
floating geometry, readable context labels, all three supported contexts, regular-tab
fallback, command dispatch without an accidental first point, and exact saved panel/tab
preferences after restart. The original drawing was untouched; visual checks used the
sample drawing and disposable geometry in a separate workspace profile. The wasm check
emitted the same three existing dead-code warnings in canvas rebuild code.
Scope: floating panels remain inside the application window. Native OS/second-monitor panels,
user-authored ribbon layouts, an in-place Text Editor ribbon, and the rest of AutoCAD's
context-specific properties/tool catalog remain gaps. No new editing engine is implied.

## Native ribbon windows and multiple monitors (2026-10-25)

Reference: AutoCAD 2027's public [ribbon documentation](https://help.autodesk.com/cloudhelp/2027/ENU/AutoCAD-LT/files/GUID-D20EF1D7-4135-48A7-B68E-65BF3BFF3D70.htm)
describes detached panels on another monitor and retention across ribbon-tab changes.
Implementation uses the existing egui/eframe immediate-viewport API and original CADCraft tools.
No new dependency, Autodesk asset, or private drawing is included.

| ID | Trigger and resulting behavior | Main files | Validation |
|---|---|---|---|
| CC-028 | In-app floating ribbon windows stopped at the application boundary. Use native desktop windows, with browser/unsupported-host fallback. Keep panel-qualified identities, command dispatch, keyboard shortcuts, Return to Ribbon and independent close behavior. Return focus to the drawing when a tool starts; hide with main-window minimization and lower panels when the application loses focus. Avoid re-detaching an already floating panel through its body footer. | `ribbon.rs`, `ribbon/floating.rs`, `ribbon/floating/native.rs`, desktop `main.rs` | Native callback and window lifecycle tests; Windows visual/OS checks below. |
| CC-029 | Application-relative coordinates could not preserve second-monitor placement. Add optional physical-pixel desktop positions while retaining embedded anchors and old-settings defaults. Capture monitor geometry/scales through the existing host, translate native positions at each viewport's current DPI, recover missing-monitor locations and validate hostile coordinates atomically. Preserve native placement when the fallback view is constrained. | `ribbon/floating.rs`, `ribbon/floating/native.rs`, `apps/cadcraft/src/{main.rs,workspace.rs}`, ribbon tests | Negative coordinates, mixed scaling, shared display edges, monitor removal, malformed inputs, legacy load and native workspace round-trip. |
| CC-030 | Native tools need identifiable coordinates and reproducible contribution evidence. Add viewport IDs to observed controls and native-window/display diagnostics to UI inspection; extend reference-translation scanning and protocol/parity records. Test the real immediate-viewport callbacks as well as fallback interactions. | `control.rs`, `i18n/tests.rs`, `tests/{native_ribbon_viewports.rs,ribbon.rs}`, `docs/{control-protocol.md,ui-parity.md,fork-changes.md}` | Native callback tool input, multiple IDs, close/hide/reopen; full checks and native evidence below. |

Validation: final combined results are recorded below.
Scope: CC-028–CC-030 concern detached ribbon panels; CC-031 adds the entire ribbon. Side palettes retain their existing in-app
left/right/floating docking. Wayland window positioning is compositor-controlled; browser hosts
embed panel windows. The existing remaining contextual-tool and authored-layout gaps are unchanged.


## Whole-ribbon floating and live reference confirmation (2026-10-25)

Live Computer Use observations of installed AutoCAD 2027: the Home-tab shortcut menu has
Undock; using it removes the docked strip and opens a narrow vertical ribbon palette with
side tabs and stacked panel buttons. Its grip menu includes Allow Docking, Anchor Left,
Anchor Right and Auto-hide. Double-clicking the grip restores the top dock. With LOCKUI
already 0 (queried and cancelled without changing it), dragging the Draw panel title
also detaches that panel. Draw remains available after switching to Insert; its upper-right
return control restores it to Home. Both floating forms therefore belong in CADCraft.
AutoCAD was returned to Home with Draw and the whole ribbon docked; no drawing was saved.
Screenshots were observed through Computer Use and are not committed.

| ID | Trigger and resulting behavior | Main files | Validation |
|---|---|---|---|
| CC-031 | The whole ribbon could not float. Add Undock Ribbon/Dock Ribbon to shared shortcut/options menus and a distinct resizable native ribbon window. Narrow windows use side tabs and stacked panel flyouts; wide windows expose the horizontal tool strip. The internal grip supports native dragging and double-click docking. Close hides the whole ribbon and RIBBON reopens it; individual close continues to return its panel. Persist and validate independent desktop geometry, default old workspaces to docked, aggregate whole/panel viewport lifecycle and reuse placement/focus handling. Extend Ukrainian coverage, workspace round-trip, native callback/fallback interaction tests and protocol/UI/hardware records. | `ribbon.rs`, `ribbon/window.rs`, `ribbon/floating{.rs,/native.rs}`, `menus.rs`, `i18n/{tests.rs,uk.tsv}`, workspace/tests and parity/control documents | Atomic malformed-input tests, legacy defaults, independent detached-panel state, native close/reopen/dock, multiple viewport identities, embedded tab/dock interaction; combined results below. |

Additional CC-031 regressions and presentation details:

- Use a vertical layout inside the compact panel scroll area. Inheriting the outer
  horizontal tab/panel layout clipped native buttons and expanded embedded windows into
  a wide strip; both presentations remain resizable.
- Wait for native position and client-size acknowledgement before saving observed geometry.
  Sizing/discard passes can render repeatedly before Windows applies a move or shrink.
  A bounded one-second UI-time window retries requests, then accepts host constraints.
  Native tests delay both position and size commands and exercise subsequent user resizing.
- A Return to Ribbon action cannot be undone by a geometry update from the same frame.
  Record the native return control for viewport-aware automation and test that race.
- Preserve accented contextual tabs, right-click selection of the tab being customized,
  minimized tab flyouts and panel-title modes in the wide floating ribbon. Test flyout
  command dispatch without expanding the ribbon.
- Paint the entire native client background with the ribbon theme, including unused
  space after resizing.

Combined validation on Windows, 2026-10-25: all six `cargo xtask ci` steps passed
(fmt, Clippy, tests, assets, layering, wasm); 2,310 tests passed, with no failures or
ignored tests. All 44 focused ribbon/native-window/palette cases passed. The final
`cargo build --workspace --bins --locked -j4` passed. Wasm emitted only the same three
existing dead-code warnings in canvas rebuild code.

Windows native checks used two physical displays (3840×2160 and 3240×2160), both at 150%.
Individual Draw and Modify panels were independently placed on the second monitor;
Line and overflow Arc dispatch, native caption dragging, close/return, main-window
minimize/restore, focus-level changes and saved placement after restart were checked.
The whole ribbon was undocked using its actual shortcut menu, returned through its grip,
and verified in compact 360×520 and wide 1000×260 client layouts at desktop position
[4200,250]. Repeated wide-to-compact requests retained that second-monitor position.
Windows' native Size menu/keyboard controls changed the client width to about 370.67
logical points and CADCraft saved that user resize. Native close hides the whole ribbon;
RIBBON restores it, preserving independent panel behavior. Restart restored whole-ribbon
position/size. Tool input reached the LINE first-point prompt without editing the sample.
Checks used the Bracket sample (46 entities, clean document) in an isolated profile;
the original project drawing was untouched.

Mixed-DPI monitor layouts, negative origins, shared edges and removed-display recovery
have synthetic regression coverage; physical monitor removal and mixed-DPI hardware were
not tested. Other native platforms were not device-tested. Browser/unsupported hosts use
embedded windows.

Remaining reference differences: whole-ribbon edge anchoring/hover auto-hide and automatic
edge-drag docking are not implemented in this arc; Dock Ribbon/internal-grip double-click
restore the top strip. CADCraft uses horizontal text tabs in the vertical palette, original
icons and its existing tool catalog, not Autodesk assets or an exact skin. Native side
palettes, additional contexts and user-authored layouts remain gaps. This is a focused
floating-ribbon improvement, not a full parity claim.

## Whole-ribbon anchoring and roll-up (2026-10-25)

Live AutoCAD 2027 Computer Use confirmed Anchor Left and Anchor Right produce a
full-height edge rail, enable Auto-hide, and expand on pointer hover. Moving away
collapses the overlay. Dragging an anchored ribbon away leaves a floating ribbon
with Auto-hide retained; floating auto-hide also rolls up away from app edges.
Turning Auto-hide off and dragging the floating grip to the application top restores
the horizontal ribbon. The reference was returned to its top-docked Home ribbon;
no drawing was saved and no Autodesk screenshots/assets are included.

| ID | Trigger and resulting behavior | Main files | Validation |
|---|---|---|---|
| CC-032 | Whole ribbon had no edge anchors. Shared options add Anchor Left/Right, checked Auto-hide, Keep Open and Allow Docking. Anchoring reserves a narrow rail; hover opens an overlay, leaving rolls up after a delay, pinning keeps it docked open, and Undock returns to a desktop window. The overlay blocks drawing/dock input. Save placement choices with legacy defaults and atomic validation. | `ribbon{.rs,/window.rs,/window/behavior.rs}`, `docking.rs`, `apps/cadcraft/src/workspace.rs`, ribbon/native tests | 52 focused ribbon/native/palette tests passed; Windows live checks and full-suite results below. |
| CC-033 | Floating whole ribbon could not roll up. Collapse to a narrow vertical caption strip, hover to expand, protect menus/keyboard/drag activity, preserve expanded size and desktop geometry independently, and support grip drag/double-click dock while collapsed. | `ribbon/window{.rs,/behavior.rs}`, `ribbon/floating/native.rs`, tests | 52 focused ribbon/native/palette tests passed; Windows live checks and full-suite results below. |
| CC-034 | Native whole-ribbon movement had no docking targets. Detect observed user moves separately from requested placement, preview app-edge targets, and dock/anchor after movement settles. Allow Docking and Ctrl suppress automatic docking. Keep monitor placement recovery independent of docking. Add Ukrainian strings, source-string scanning, protocol and parity records for CC-032–034. | ribbon window/native modules, `i18n/{tests.rs,uk.tsv}`, protocol/UI/hardware docs, tests | 52 focused ribbon/native/palette tests passed; Windows live checks and full-suite results below. |

CC-032–034 regression details: a Windows Undock check exposed a reentrant egui
context read while converting an anchored grip position to desktop pixels. Read the
scale before entering the input closure; cover Undock with native root coordinates
in the interaction harness. Pin controls also need the body background hit region to
start below the anchor header. Context-menu activity holds the overlay open; pin
clicks cannot supply points to a running drawing command. Treat native roll-up
transitions as requested placement so decoration/size changes do not count as user
docking gestures. Embedded floating ribbons explicitly resize for roll-up while
ordinary embedded windows retain their previous minimum size.

Windows on-screen validation used the clean Bracket sample in an isolated profile.
Computer Use confirmed left/right rail hover, Keep Open, the placement shortcut menu,
anchored Undock, floating strip hover/roll-up on the second monitor, native caption
drag previews and top/left/right snapping. A caption drag remained floating with
Allow Docking disabled, then anchored after re-enabling it. Control inspection
confirmed saved anchor/autoHide and independent [360,520] expanded size at physical
[4200,250], even while the native strip was only 48 physical pixels wide at 150%.
Clean quit/restart restored both an anchored layout and the second-monitor floating
roll-up layout. Every old test process was closed before the next launch; the
original drawing was never opened or edited by these checks. The final test build
is left with its normal top ribbon and a clean sample drawing.

Limits: docking follows 300 ms of stable observed native movement; exact native
OS drag-cancel/lock behavior and continuous pointer capture from an anchored grip
into a new OS window are not implemented. The Undock gesture creates the floating
window at the gesture position, where its native caption/grip continues movement.
Ctrl bypass has native callback regression coverage, rather than a held-key desktop
check. Anchored hover/roll-up has interaction coverage; browser device checks remain open, and automatic edge
snapping currently applies to native floating windows. Other native platforms and
physical mixed-DPI layouts were not device-tested. This is a focused behavior match,
not a complete AutoCAD skin or full ribbon parity claim.

Final automated validation (Windows, 2026-10-25): all six `cargo xtask ci` gates
passed (fmt, Clippy, tests, assets, layering, wasm), with 2,319 tests passed, zero
failed and zero ignored. The focused ribbon/native/palette run passed all 52 cases.
The catalog checks validate the empty context column and all four new Ukrainian
labels; the workspace round-trip includes autoHide and allowDocking. Wasm emitted
only the same three existing dead-code warnings in canvas rebuild code.
The final `cargo build --workspace --bins --locked -j4` also passed. Its copied
Windows executable was reopened only after the older test instance exited; launch,
top-docked ribbon and clean sample state were checked in Computer Use/control.

## GUI inventory and paper-space/status controls (2026-10-25)

| ID | Trigger and resulting behavior | Main files | Validation |
|---|---|---|---|
| CC-035 | Establish a first-level Windows GUI inventory from live AutoCAD 2027 ribbon/status inspection and CADCraft runtime/source comparison. Record 31 surfaces, evidence levels, uninspected areas and prioritized follow-ups; distinguish visible commands from tested behavior and preserve the original reference drawing. | `docs/autocad-2027-gui-inventory.md`, UI parity/gaps records | Home/Insert/Annotate/Parametric/View/Manage/Output/Layout/Add-ins/Collaborate/Express Tools and status customization observed. Coordinates visibility toggled and restored; no save or cloud action. |
| CC-036 | Opening a paper layout left the ribbon on unrelated tools. Add a transient Layout tab with Layout, Layout Viewports, Space and Plot panels using registered commands. Keep Layout available alongside Hatch/Table contexts, permit explicit context choice, retain the saved regular tab, and preserve detached contextual panels while their context remains available. Reuse docked/embedded/native ribbon renderers. | `ribbon{.rs,/context.rs,/floating.rs,/window.rs}`, ribbon tests, Ukrainian catalog | Paper/model transitions, regular-tab restoration, MVIEW dispatch, coexisting Table/Layout contexts, floating panels and disabled-context recovery; results below. |
| CC-037 | Fixed status controls could not be hidden individually. Add a rightmost customization menu, background shortcut menu and Panels recovery entry. Persist validated per-item visibility for drafting controls, coordinates, space/scales and existing workspace/settings buttons. Keep Restore All outside the scroll area and visible when all items are hidden. Hidden coordinates do not suppress transient status/error messages. Visibility does not alter drafting state. | `chrome{.rs,/status.rs}`, `lib.rs`, `menus.rs`, `panel_menu.rs`, native workspace, status tests, i18n/catalog/protocol | Atomic invalid input, legacy defaults, visibility persistence/reset, native workspace round-trip and LINE input protection; results below. |
| CC-038 | Escape while a popup was open also canceled a running drawing command. Let a popup own its first plain Escape before command-line keyboard processing; a later Escape can cancel normally. This protection applies to menus in root and native ribbon viewports. | `cmdline.rs`, `lib.rs`, status interaction regression | Open status customization while LINE runs, hide Grid without changing GRIDMODE or adding a point, dismiss the popup and continue the line; results below. |

The inventory is not a claim of complete AutoCAD parity. Next priorities include inline
layer/property/style controls, attribute-format/visual fixture checks, replacing the
workspace no-op, native side palettes and quick-access customization. CADCraft's existing
engine does not provide every visible AutoCAD workflow; unsupported tools were not added
as decorative buttons. No Autodesk screenshots/assets or private drawing files are part
of this contribution. Layout currently exposes supported 2D commands, not 3D view creation
or standards controls. Status visibility retains existing defaults; a fully adaptive
narrow-window overflow and arbitrary saved workspace presets remain follow-ups.

Windows visual validation used a clean Bracket sample in an isolated profile after
closing the previous build. Computer Use confirmed the blue Layout tab and its four
panels in Layout1, then its disappearance and Home restoration in Model. The rightmost
status customization menu showed checked visibility choices and a fixed Show All action.
With LINE waiting for its first point, hiding Grid removed its status button; Escape
closed the menu while LINE remained active, and a second Escape canceled normally.
Control inspection confirmed hiddenStatusItems, command state and an unchanged clean
drawing. Visibility was reset and the sample returned to Model. The original DWG was
not opened or modified. Focused ribbon/status/native tests passed all 50 cases; the
Windows desktop build passed. General label clipping at this window width remains
tracked by GI-31; paper-layout tools do not add unsupported 3D functionality.
Final validation (Windows, 2026-10-25): `cargo xtask ci` passed all six gates
(fmt, Clippy with warnings denied, tests, assets, layering and wasm). The full run
passed 2,323 tests with zero failures and zero ignored. Wasm retained only the three
existing dead-code warnings in canvas rebuild code. The focused 50-test run and
`cargo build --workspace --bins --locked -j4` also passed. The initial formatting
check was corrected with the repository formatter before the successful full run.

## CC-039 — ribbon layer selector (2026-10-25)

Problem: Home / Layers offered actions but no inline layer selector. Add a bounded,
scrollable selector with current-layer/selected-object semantics, a blank mixed-layer
value, original layer-state icons and color swatches. Choosing a name with no selection
runs layer.current; choosing it with a selection runs properties.set with those handles,
leaving CLAYER unchanged. Layer-state buttons use layer.set. Preserve the four existing
actions in two compact rows within the existing panel width so later panels remain reachable.
No workspace schema or drawing-format change. Disable object reassignment when the selection
contains a locked-layer object; state controls can still unlock it first.

Files: ribbon.rs, ribbon/layer.rs, tests/ribbon.rs, tests/ribbon_layers.rs,
reference translation/catalog scan, AGENTS.md and inventory/parity/contribution records.

Reference evidence: Computer Use in AutoCAD 2027 on a new disposable Drawing1 with two
synthetic LINEs on CC_A and CC_B. The dropdown listed layers with bulb/freeze/lock and color;
choosing CC_A without selection made it current. Selecting the CC_B line displayed CC_B;
choosing 0 edited that line, and clearing selection restored CC_A. Selecting both lines
showed a blank mixed value. Undo reported CHPROP for the layer reassignment (after the
separate Select All undo group). No drawing was saved or exported, and the private reference
drawing was not edited. Some inactive canvas frames did not repaint until selection;
these checks establish control semantics, not rendering/plot fidelity.

Dependencies and reversal: requires the existing ribbon foundation (CC-013–019) and
reuses existing layer.current/layer.set/properties.set engine behavior. First-Escape
protection during LINE relies on CC-038. Its tests also cover detached panels from CC-025.
Reverting CC-039 as a unit removes only this ribbon selector, compact Layers tool arrangement
and its tests/translations/docs; the earlier Layers palette, Properties editor,
engine commands, status controls and Layout tab continue unchanged. No later feature
currently depends on this module. Future consumers must declare that dependency here.
Historical multi-ID reversal limits are documented above. Color/linetype/style ribbon
selectors remain separate follow-ups, so they are not coupled to this commit.

Focused validation: all 51 ribbon/layer/native tests passed; the Windows desktop build passed.
Computer Use confirmed current-layer selection, selected-object layer display and assignment,
blank mixed state, unchanged CLAYER, and a working selector in the native detached Layers
window. Control inspection verified object layer 0/current CC_A and Undo back to CC_B.
Returned the panel to the ribbon; saved only the synthetic CADCraft fixture outside the repo
and returned to the clean Bracket sample. The previous build exited before launch.
The mixed locked/unlocked selection policy is conservative (assignment disabled until unlocked);
that exact AutoCAD variant and the color-swatch editing gesture remain unverified/follow-up.
Full gate results follow below.

Final source validation: all six cargo xtask ci gates passed (fmt, Clippy with warnings
denied, tests, assets, layering and wasm). There were 2,327 tests passed, zero failed and
zero ignored. This includes the selector right-click / Float Panel regression added after
on-screen testing found the missing context-menu forwarding. Layer-list allocation/sorting
now occurs only while its popup is open. The initial test/lint compilation issues were
corrected before this complete final run. Wasm retains the existing three canvas rebuild
dead-code warnings. A reverse-apply dry run of the staged patch passed without modifying
files; baseline fc0c091 remains the prior published version. Reversal still requires the
dependency review described above and recording the inverse operation.
The final desktop build also passed. After confirming all prior test processes had
exited, reopened that executable and used Computer Use to right-click Ribbon Layer,
choose Float Panel, inspect its native Layers window and click Return to Ribbon.
The selector and four actions returned to Home; inspection showed zero detached panels
and a clean Bracket sample. This final binary is left open for review.
## CC-040 — selection-aware ribbon color control (2026-10-25)

Home / Properties gains an inline color selector and swatch. With no selection it
sets the color for new objects through color; with a selection it runs properties.set
for those handles, leaving the current color unchanged. Mixed logical values show a
blank field. ByLayer remains one logical value across different layer colors; its
swatch resolves against the first selected object's layer, as observed in AutoCAD.
Exact current RGB values are read from CECOLOR_RGB rather than their nearest index.
The three existing Properties actions remain below the selector in the same panel width.

The popup offers inherited and standard indexed colors with original swatches and a
validated index/RGB/hex input. Its bounded scroll area keeps these choices reachable
in the small detached native panel. Locked-layer selections cannot be recolored here.
Right-clicking the selector preserves the panel Float/Return and visibility menu.
No drawing format, workspace schema or engine command behavior changes.

Reference: Computer Use in AutoCAD 2027, disposable Drawing1 with two synthetic LINEs.
Setting Blue without selection changed the current color. Selecting two ByLayer lines
on red/green layers still displayed ByLayer with the first layer's red swatch. Selecting
Green changed both objects. Clearing selection restored Blue. Changing just one line to
Red and selecting both showed a blank color field. Undo reported CHPROP after undoing
the separate selection group. More Colors opened Select Color with Index Color, True
Color and Color Books tabs; True Color exposed RGB entry. Canceled without applying.
The private reference drawing was not edited. No reference screenshot or asset is committed.

Dependencies/reversal: depends on the ribbon/display foundation CC-013–024. Popup-first
Escape protection relies on CC-038; detached/native verification uses CC-025/028. Reuses
existing color/properties.set commands and original layer swatch painter. No dependency
on the CC-039 layer-selector module: the two selectors share panel placement only.
Revert this commit as a unit to remove the new color UI, its compact Properties arrangement,
tests, translations and records; Layers, the earlier Properties palette and engine color
commands continue unchanged. Check later entries for consumers before reverting.

Remaining visual/function differences: compact index/RGB text entry replaces the full
Select Color dialog; HSL editing, color-book libraries and the expanded visual color grid
remain follow-ups. This does not claim full Properties/ribbon parity.
Validation results are recorded below after the final run.

Focused validation: 53 ribbon/color/native tests passed and the Windows desktop build
passed. Computer Use verified current Blue, selected ByLayer then Green, blank mixed
Green/Blue state and selector right-click / Float Panel. In the native Properties window,
scrolled to the compact custom editor, entered #0c2238 and clicked Apply Color. Control
inspection confirmed both selected lines were exactly 12,34,56 and the current color
remained Blue. Undo restored the separate Green/Blue object colors. The old CC-039 build
was verified clean and exited before the CC-040 executable was opened.
The native Return to Ribbon button restored the selector and all three actions; inspection
reported zero detached panels. Saved only the synthetic two-line drawing outside the repo
and switched back to the clean Bracket sample. A reverse-apply check of the staged feature
patch passed without changing files; its final documented reversal scope is unchanged.
Final validation: all six cargo xtask ci gates passed (fmt, Clippy with warnings denied,
tests, assets, layering and wasm): 2,333 tests passed, zero failures and zero ignored.
Wasm retains the same three pre-existing canvas rebuild dead-code warnings. The final
Windows executable passed on-screen current/selected/mixed/native custom-color checks.
A fresh disposable AutoCAD Drawing2 repeated current Blue / selected ByLayer / selected
Green / cleared-selection Blue with a synthetic line; the original drawing stayed untouched.

## CC-041 — selection-aware ribbon linetype control (2026-10-25)

Home / Properties gains a second inline selector for loaded linetypes, with original
pattern previews. No selection changes CELTYPE through linetype; selected handles change
through properties.set, leaving CELTYPE unchanged. Mixed logical types show a blank field.
ByLayer and ByBlock remain logical values. Locked-layer selections cannot be reassigned.
The bounded scrolling popup supports small native floating panels and loading CADCraft's
original standard library without silently setting the loaded type current or assigning it.
Properties Inspector and Match Properties remain below the fields; Quick Select is in the
panel More menu. Existing panel width is unchanged, preserving the other Home panels.

Reference: Computer Use in AutoCAD 2027 on fresh disposable Drawing2 with a synthetic LINE.
No selection: choosing Continuous displayed it as the current linetype. Selecting the older
ByLayer line displayed ByLayer. Choosing ByBlock then returning focus to the canvas committed
the edit; after clearing selection, Continuous returned, and reselecting the line showed
ByBlock. Choosing Other opened Linetype Manager, whose current type remained Continuous;
its Load button opened a separate available-types list. Canceled both dialogs without loading
or saving reference data. No private drawing or Autodesk asset was modified or committed.
An immediate Escape while the reference linetype field retained focus canceled the pending
choice; CADCraft currently commits a clicked choice immediately. Preview-until-focus-loss,
full manager/filter/reload/file UI and exact dialog appearance remain follow-ups.

Dependencies/reversal: uses ribbon/display foundation CC-013–024 and popup-first Escape
protection CC-038; native verification uses CC-025/028. This Properties layout builds on
CC-040's color-row placement: reverting CC-040 later requires adapting this layout and its
geometry tests, or reversing CC-041 first. Its linetype module does not call the color or
layer selector modules. The shared Layers dash painter is only made crate-visible, with
no change to its behavior. Revert this commit to remove the linetype row, load UI and tests,
return Quick Select to its earlier direct button, and restore CC-040's compact arrangement.
The CC-040 color control, CC-039 Layers selector and earlier engine commands remain intact.
Tests and final build/GUI/full-gate results follow below.

Focused validation: all 58 ribbon/color/linetype/native-window tests passed, including
scrolling a library item into view before loading it, current-vs-selected edits and Undo/Redo,
active LINE isolation and first Escape, locked selection, Float/Return context routing,
retained Quick Select access and bounded native popup interaction. Test-query ambiguity
and an offscreen test click were corrected before this complete focused run.
Windows desktop validation: the previous build exited cleanly before this executable
opened. Computer Use confirmed DASHED as current, selected ByLayer, selected ByBlock,
unchanged current DASHED and Undo back to ByLayer. Right-clicking the linetype selector
opened the existing panel menu and detached Properties. Both selectors fit the native
window. Scrolled to Load Linetype, expanded the original library and loaded CENTER;
inspection confirmed it was loaded while the current type remained DASHED. The native
More menu launched the real Quick Select dialog, which was canceled. An existing native
window stacking limitation remains visible: the floating panel can overlap an in-window
dialog; address modality/stacking in the broader native-window follow-up.

Final validation: all six cargo xtask ci gates passed (fmt, clippy, tests, assets,
layers, wasm), with 2,338 tests passed and none failed or ignored. Computer Use
confirmed Return to Ribbon restored the Properties panel. The disposable fixture
was saved outside the repository and the clean Bracket sample restored; both documents
are clean. Reverse-apply validation is performed before committing this arc.

## CC-042 — Inline dimension-style control (2026-10-25)

Adds a real style selector to Annotate / Dimensions. With no selection it reads
DIMSTYLE and changes the drawing default via dimstyle.current. With selected
DIMENSION objects it shows their common stored style, or a blank mixed value,
and changes only those dimensions through properties.set. Non-dimension objects
are not edited. Locked dimensions disable assignment. Manage Dimension Styles
opens a new basic dimension-style manager; the original drawing commands remain reachable in
the panel and More menu. The same bounded popup works in detached panels.

Reference: Computer Use in AutoCAD 2027, disposable Drawing2. Annotate's dimension
style gallery offered Standard and Annotative plus Manage Dimension Styles.
Created an Annotative linear dimension, set current Standard, selected the
existing dimension and saw Annotative. Changed it to Standard, reselected to
confirm, then changed it back to Annotative. After Escape and a fresh repaint,
the unselected field showed current Standard. No Autodesk patterns, fonts,
assets, files or screenshots are copied into the repository.

Dependencies/reversal: uses the shared ribbon foundation CC-013–024 and the
inline-row layout introduced in CC-039, plus popup-first Escape CC-038; native
verification uses CC-025/028. It calls existing engine commands and does not
modify their behavior. Revert this commit to remove only this selector, its
translations, manager routing and tests, restoring the earlier Dimensions button arrangement.
CC-039–041 controls remain intact. No other arc currently depends on CC-042.

Remaining differences: this is an original compact list rather than the reference
thumbnail gallery; dimension-style preview tiles, full tabbed Modify/Override/Compare UI, per-dimension override UI,
annotation scaling controls and text/leader/table-style selectors remain separate
work. CADCraft preserves the existing selection when applying; the reference
cleared selection after the style command. Modeless/native dialog stacking is
tracked separately in CC-041. Validation results follow below.

The manager gap was exposed by the interaction test: the existing Dimension Style
launcher had no UI route. This arc adds routes for dimstyle, dimstyle.dimension,
d, dst and ddim, plus a manager using the established style-manager surface.
It lists styles, creates a named style from the current style, sets current, and
edits text height/style, arrow size, overall/linear scale, extension offset/length
and text gap through the existing dimstyle command. These immediate edits are
undoable, as in the other CADCraft style managers. Programmatic JSON stays dialog-free.
AutoCAD's live manager confirmed current Standard after the selected dimension
was changed to Annotative; Modify exposes a larger tabbed settings editor with
preview. That broader UI and staged OK/Cancel editing remain follow-ups.

Focused validation: 54 ribbon/native/style-selector interaction tests passed
(37 ribbon, 13 native, 4 dimension-style), plus the manager workflow test for
alias routing, dialog-free JSON, creation, Set Current and separate Undo steps.
The initial test API/query errors were corrected; the failed manager-opening
assertion revealed the missing production route described above.
Windows build passed. The prior CC-041 process exited with all drawings clean
before CC-042 launched; no other CADCraft build remained. Computer Use verified
current Review vs selected Standard, assignment to Review with visible text update,
and Undo back to Standard while current remained Review. Right-click Float Panel,
native style selection, bounded popup and Return to Ribbon all worked. The existing
Styles launcher opened the new manager. Edited non-current Review text height
3 to 4 using its field, verified stored height 4/current Standard, and Set Current
changed the current style to Review. Undo restored current Standard then height 3.
Saved only the synthetic fixture outside Git. Full-gate results follow below.

Final validation: the first full run caught two newly added Ukrainian catalog rows
missing their empty context column. Corrected the three-column format and reran
`cargo xtask ci`: all six gates passed, 2,344 tests passed, zero failed or ignored.
Restored the clean Bracket sample; both open drawings are clean and no floating
panel or manager remains. The synthetic fixture stays outside Git. The staged
inverse patch was checked before publication to verify this arc can be reversed.

## CC-043 — selection-aware ribbon text styles

Trigger: Annotate / Text lacked the reference application's inline style selector,
and properties.set could not assign a style to existing TEXT or MTEXT.
Live Computer Use in AutoCAD 2027 on disposable Drawing2 confirmed current style
Standard, current selection Annotative, assignment of Standard and back to Annotative,
and current Standard after deselection. Manage Text Styles opened a manager showing
current Standard. Properties showed the selected MTEXT Style Annotative / Annotative
Yes; assigning Standard changed those to Standard / No and preserved height 0.2000,
rotation 0, contents and defined width. No Autodesk files or artwork were read/copied.

Change: Annotate / Text has a bounded, sorted style list, current TEXTSTYLE without
selection and common/blank mixed style for selected standalone TEXT/MTEXT. Assigning
a style affects only those text entities and leaves the drawing default unchanged.
Locked text disables assignment; unrelated-only selections cannot change the default.
Manage Text Styles opens the existing real manager. Compact text tool buttons keep
the original tools reachable, and selector context menus retain Float/Return behavior.
The properties.set textStyle field validates a loaded name before edits, canonicalizes
case, assigns TEXT/MTEXT style and adopts its annotative setting through the existing
annotation-scale helper. Explicit annotative in the same JSON request takes precedence.
Text content, geometry and per-object formatting remain intact. Undo/Redo and normal
serialization use the existing command pipeline. Added original labels and Ukrainian
translations with catalog coverage.

Files: engine cmd/props.rs and tests/text_style_properties.rs; ui-egui ribbon.rs,
ribbon/textstyle.rs, i18n catalogs/tests, tests/ribbon_textstyles.rs and
native_ribbon_viewports.rs; this ledger, GUI inventory and UI parity journal.

Dependencies/reversal: shared ribbon foundations CC-013–024/039, popup-first Escape
CC-038 and the inline dimension layout CC-042. Revert this feature commit to remove
only the text selector, textStyle command field, localization and tests and restore
the earlier Text tool arrangement. Dimension/color/linetype/layer selectors and
existing text manager remain intact. Later callers of the new textStyle field would
need adaptation before reversal; none are currently recorded. This arc changes the
engine command schema, so downstream integrations must stop using that field on revert.

Remaining differences: original compact names rather than preview gallery; font picker,
full text-style manager controls/preview, fixed-height and style-effect inheritance
comparisons remain open. Attribute definitions/attached attribute text, leaders, tables
and nested block content are intentionally separate work items; they are not rewritten
by this control. CADCraft retains selection after assignment, while reference style
commands cleared it in the earlier checks. Validation results follow below.

Focused validation: 20 tests passed (two engine style/annotation/atomic-validation
and DXF round-trip tests, four text-selector interaction tests, 14 native ribbon
regressions). Windows build passed. All drawings were clean before CC-042 exited;
no stale CADCraft process remained before CC-043 started. Computer Use verified
current Review with existing Standard text untouched, selecting MTEXT showing Standard,
Review assignment narrowing the visible text through its width factor, and stored
style Review / annotative Yes with scale 1:1 while contents/position/height/width/
rotation stayed intact. Ctrl+Z restored Standard/non-annotative text. Native Text
panel popup fit; selection of current Standard and Return to Ribbon worked. Manage
Text Styles opened the existing manager. Saved only the synthetic drawing outside
Git and restored the clean Bracket sample. Full-gate results follow below.

The first full CI run stopped in the existing CLI connect_to_closed_port_fails test:
the spawned client unexpectedly exited successfully when a connection failure was expected.
A focused rerun passed all four connection tests without source changes. Port reuse
is a suspected test-fixture race, not proven from the original failure (the address
was not printed). Track deterministic port isolation as a separate reversible arc;
no connection production/test code is changed in CC-043. Rerunning the full gate.

Final validation: the unchanged full rerun passed all six cargo xtask ci gates
(fmt, clippy, 2,351 tests with zero failures/ignored, assets, layers and WebAssembly).
The staged inverse patch passed git apply --reverse --check before publication.

## CC-044 — deterministic CLI connection-failure fixture

Trigger: the first CC-043 full run intermittently failed the existing
connect_to_closed_port_fails test because its client exited successfully. All four
connection tests passed unchanged on rerun; the subsequent full CI also passed.
The fixture bound and then released an ephemeral listener before starting the
client, leaving its destination available to other tests/processes. Port reuse is
a plausible explanation, not a proven diagnosis of that original failure.

Change: the negative startup test now connects to loopback destination port zero,
which cannot designate a listening TCP endpoint (binding to zero allocates a
nonzero port). It sends both initialize and a drawing-edit request and asserts
nonzero exit, no protocol replies and the connection error. The positive test
still uses a real ephemeral listener and verifies remote request routing. This
removes the released-port race from the negative fixture without changing the
application's connection behavior or adding dependencies.

Files: apps/cadcraft-cli/tests/mcp_connect.rs and this ledger. No UI or drawing
behavior changes, so no replacement build or AutoCAD interaction is needed.
Dependencies/reversal: independent of CC-043 and other UI arcs. Reverting this
commit restores the former test fixture only; no downstream command/API changes.
Validation results follow below.

Windows endpoint check using the existing CLI build: loopback destination port zero
returned exit 1, zero stdout bytes and connection error 10049 (address invalid in
its context), confirming it cannot silently serve headless requests on this host.
Focused/full-suite validation of the updated test follows below.

The four connection regressions passed in 20 consecutive runs (80 test executions),
including simultaneous positive endpoint routing and negative startup checks. Clarified
the CC-043 failure record: the failed assertion established a successful client exit,
not the identity of a listening server. No production code changes were needed.

Final validation: all six cargo xtask ci gates passed (fmt, clippy, 2,351 tests
with zero failures/ignored, assets, layers, WebAssembly). The inverse-patch dry run
passed before publication. This test-only arc requires no new GUI build.

## CC-045 — per-reference attribute Text Options

Trigger: CC-005 exposed values but no text-format controls. Live Computer Use in
AutoCAD 2027 on a disposable two-attribute Equipment block confirmed Attribute,
Text Options and Properties tabs. Text Options exposes style, justification,
height, width factor, rotation, oblique, Backwards/Upside down, annotation and
boundary width (disabled for single-line attributes). Height 3 to 4 previewed
before Apply; Apply then Cancel retained 4. A later Top center change was discarded
by Cancel. Undo restored the original height after the cancelled-preview undo group.
The CADCraft-generated fixture was never saved by AutoCAD. No Autodesk output,
assets/fonts or private drawings are included.

Change: ATTEDIT accepts formats keyed by attribute tag, with loaded textStyle,
height, widthFactor, rotation/oblique in degrees and justification. All formats
validate before any value edit. Names match without case, styles use canonical
loaded names, and requested-tag indexing scans a reference once with bounded input.
Unknown/ambiguous/duplicate tags, unknown fields, invalid numeric ranges, constant
or multiline formatting and locked/noneditable targets fail without partial edits.
Normal justification switches retain the existing alignment anchor, including its
Z coordinate. Actual placement, unedited formatting, block definition, other attributes/instances,
properties and modes are preserved unless explicitly edited. Values and formats
share one command/Undo/Redo operation and existing DXF serialization.

The editor retains the existing value fields in Attribute and adds Text Options
with an attribute target, style/justification selectors and numeric fields. Locked
references are read-only. Invalid input keeps the editor open with an explanation.
Apply tracks explicit draft changes, so unchanged drafts do not replay prior edits
after external edits; Undo clears selection and closes the editor as before. Cancel
discards unapplied values/options. No new file-format
fields or migration are introduced. New editor labels currently use English, as did
the existing attribute editor; full localization remains open.

Files: engine cmd/blocks.rs, blocks/attedit.rs, blocks/attedit_format.rs and
tests/attribute_text_options.rs; ui-egui attributedlg.rs and
attributedlg/text_options.rs and attributedlg/tests.rs; this ledger and the GUI/UI parity records.

Dependencies/reversal: CC-005 attribute dialog and CC-003 attribute import alignment.
No dependency on CC-043's standalone-text properties.set field. Revert this feature
commit to remove formats support and the Text Options editor/tests and restore the
previous value editor. Later formats callers must be adapted first. Existing drawing
fields are reused; reverting code does not undo edits already saved in drawings.
The value command also now refuses locked or out-of-working-set references.

Remaining differences: Apply-time editing rather than live preview; per-attribute
Backwards/Upside-down flags and annotation context, multiline formatting/boundary
width and a Properties tab require separate support. New Align/Fit placement needs
two points and is disabled; existing modes remain unchanged until explicitly switched
to an ordinary justification. Existing per-object fields are preserved on style change;
complete fixed-height/style-effect inheritance remains a comparison item. Font
appearance depends on installed fonts. Validation results follow below.

Documentation also corrects the CC-043 UI-journal row to retain its three-column
layout; no CC-043 behavior changes.

Compatibility refinement: the first full run passed fmt/clippy and stopped at an
overly strict round-trip assertion. The existing writer intentionally computes a
justified attribute's derived group-10 baseline while preserving its group-11
placement. The revised test checks every other field and exact rendered primitives,
vertices, triangles and bounds, permitting only that derived XY normalization.
Source review then found that a small metric edit of an imported justified attribute
could leave the cached baseline within the writer's font-substitution tolerance.
A new regression failed before the fix: height 5 to 5.5 exported the old baseline
(103.52072594216368, 198.56859045531334), rather than the scaled-offset result
(103.62638837486627, 198.0522444200878). Formatting now invalidates that derived XY
cache for ordinary justified text so existing DXF/DWG export recomputes it; the
actual alignment anchor and Z fields remain intact. Align/Fit's two placement points
are preserved. No writer tolerance or import behavior was changed.

The editor tests were also corrected to respect existing Undo selection clearing:
Undo closes the editor when its selected reference disappears. An external ATTEDIT
while selection remains now tests stale-draft protection; subsequent Undo checks
prove a no-op Apply adds no undo record. Unit tests are extracted to
ui-egui/src/attributedlg/tests.rs; no public UI API is added. Additional regressions
cover oversized requests, ambiguous tags, nonzero-Z anchors and loaded Align/Fit
points. Focused rerun/full-gate and Windows GUI results follow below.

Focused rerun passed 1,078 tests: workspace library/binary unit tests plus seven
attribute engine regressions, including all five attribute-editor interaction tests.
The previously failing baseline-export regression passed after cache invalidation.
The numeric interface accepts finite positive height <= 1e12 and width factor <= 100,
rotation within +/-1e9 degrees and oblique within +/-85 degrees; a request is limited
to 4,096 format tags. These are command validation bounds, not a claim that every
AutoCAD numeric-expression syntax is supported. The older CC-043 app had three clean
documents, exited normally, and no CADCraft process remained before the new GUI build.
Full quality gates and live Windows checks remain pending.

Windows verification: cargo build --workspace --bins --locked -j4 passed. Live GUI
changed DEVICE_TYPE from height 3/Left to height 4/Top center using Text Options,
then Apply. The title centered on the vertical guide with its top at the horizontal
guide; DEVICE_ID stayed height 3/Left/Baseline. Cancel retained the applied change,
and one GUI Undo restored original placement. Control-channel Redo prepared export.
CADCraft saved and reopened synthetic DXF and DWG: the title retained height 4,
center/top and anchor (0,15,0), and the other attribute remained unchanged.
AutoCAD 2027 opened the CADCraft-produced DWG with the same installed Arial font:
the title's top and center matched the green placement guides, and the ID retained
its lower left-baseline position. AutoCAD did not save the drawing. No font, private
file or Autodesk output is included.

Separate follow-up observations: direct filled-TrueType attribute double-click did
not open the editor on this attribute-only fixture in either application's exported
DWG check; ribbon-based editing was verified in CADCraft. This is an unconfirmed
picking/fixture issue, not a claimed CADCraft-only regression. Closing the editor
also briefly started a crossing-selection rectangle beneath the old Cancel button;
Escape cleared it. General dialog-to-canvas input isolation needs its own fix/test.
The final full quality-gate run is in progress; results will be appended.
Additional live check: reopening the saved DXF populated Standard, Top center,
height 4, width factor 1 and zero angles. Entering NaN for height and pressing OK
kept the editor open with a finite-number error; the drawing stayed unchanged.
Escape discarded the invalid draft. The analogous atomic combined value/error
behavior is covered by the editor regression tests.

The matched-file check also reproduced a separate pre-existing value-only export
problem: POWER ADAPTER to POWER ADAPTOR retained baseline X -24.511247443762777,
where recomputing for that same edited text gave -24.82072256305385. The placement
anchor stayed (0,15,0). This affects value edits independently of the new formatting
controls and is the next separate regression/fix arc; CC-045 does not claim to fix it.

Final validation: cargo xtask ci passed all six gates (fmt, Clippy, tests, assets,
layers and wasm): 2,361 tests passed, none failed or ignored. The final staged
inverse patch passed git apply --reverse --check. The Windows build, editor GUI,
invalid-input check, synthetic DXF/DWG reopen and AutoCAD DWG guide comparison
above complete this feature arc. Remaining differences and independently reproduced
follow-ups stay explicitly open.

## CC-046 — refresh justified attribute placement after value edits

Trigger: the CC-045 matched-file pass reproduced a pre-existing small value-edit
export defect. Changing POWER ADAPTER to POWER ADAPTOR retained baseline X
-24.511247443762777 instead of recomputed -24.82072256305385 with the same Arial,
height 4 and Top center anchor (0,15,0). The renderer used the anchor correctly,
but the saved baseline could remain within the writer's font-substitution tolerance.

A standalone synthetic regression failed before the fix: a rotated/scaled title
changed to POWER ADAPTER. exported baseline (57.86231594830153, 203.38460969082652)
instead of (56.73648292338176, 202.73460969082652), independently calculated from
font placement. No private drawing, external font or AutoCAD-produced file is used.

Change: the shared attribute-value setter invalidates the ordinary justified text's
derived baseline XY only when its value actually changes. JSON ATTEDIT (the dialog
route), interactive ATTEDIT and both global -ATTEDIT forms use it. The existing
DXF/DWG writer then recomputes the baseline without changing its font tolerance.
No-op edits retain imported font placement; Left/Baseline and Aligned/Fit placement
points remain intact. The actual alignment point and insertion/alignment Z values
are preserved. Legacy changed counts, tag matching and command syntax are retained.

Files: engine/src/cmd/blocks/attedit.rs; engine/tests/attribute_value_baseline.rs;
this ledger and the GUI/UI parity records. Regression checks cover all four value
routes, both saved formats, Undo/Redo, nonzero-Z anchors, other attributes/instances
and definitions, no-op font offsets and Left/Aligned/Fit point preservation.

Dependencies/reversal: uses the pre-existing attribute model and baseline-cache
convention from upstream import/export handling. It does not require CC-045 formats
or its editor UI: test fixtures initialize existing document fields directly.
Revert this commit to remove only value-change cache refresh and its tests.
CC-045 formatting cache refresh remains separate. Both commits touch attedit.rs;
reversing CC-045 first may need context resolution that retains CC-046's value
setter and callers. No file-format migration; reverting code does not undo values
already saved. Other value-changing tools and style-definition changes are outside
this narrowly reproduced fix and require their own checks. Validation is pending.

Focused validation passed 1,081 tests (workspace library/binary tests and both
attribute integration files), none failed or ignored. All three new regressions
passed, including the before-fix failure, all four edit paths and DXF/DWG checks.
The Windows workspace binary build passed. The older GUI had seven clean synthetic
documents, quit normally and left no CADCraft process; this was checked again
immediately before the new build launched with an isolated workspace profile.

Live Windows verification used the imported Arial/Top-center fixture from CC-045.
The Attribute tab changed POWER ADAPTER to POWER ADAPTOR, Apply updated only that
attribute, and Cancel kept the applied result. CADCraft saved/reopened synthetic
DXF and DWG: both now store baseline X -24.82072256305385, Y 11 with unchanged
height 4, center/top and anchor (0,15,0); DEVICE_ID stays ADPT-01, height 3,
Left/Baseline. AutoCAD 2027 opened the new CADCraft DWG and fitted it to extents:
the edited title remained top-centered on the green guides and the lower ID stayed
at its original left anchor. AutoCAD did not save the fixture. No Autodesk output,
font, screenshot or private drawing is committed. Final full CI is in progress.

Additional AutoCAD readback: Insert > Edit Attribute and a pick on the unchanged
lower ID opened Enhanced Attribute Editor for the exported DWG. Selecting DEVICE_TYPE
confirmed POWER ADAPTOR and Text Options Standard/Top center/height 4/width 1/zero
angles. The title itself was not picked at the tested point; this narrows the earlier
unconfirmed picking observation and does not establish a general DWG editing failure.
Per-attribute Properties controls were inspected separately for a future arc; they
are not part of CC-046.

Final validation: cargo xtask ci passed all six gates (fmt, Clippy, tests, assets,
layers and wasm), with 2,364 tests passed, none failed or ignored. The complete
staged inverse patch passed git apply --reverse --check. The three new regression
tests, Windows build/editor edit, saved DXF/DWG checks and AutoCAD native editor
readback above complete this narrowly scoped value-edit alignment fix.

## CC-047 — isolate attribute-editor input from the drawing

Trigger/reference: AutoCAD 2027 Enhanced Attribute Editor remains open when its
underlying drawing is clicked, without starting a selection. A full CADCraft UI
regression failed before this fix because the same outside click set pending_window.
A separate Cancel-over-canvas check already passed before the fix; Cancel alone is
not recorded as a confirmed defect.

Change: a modal input shield surrounds the existing attribute window. Outside
clicks do not dismiss it or reach the drawing/ribbon. Drawing shortcuts, command
typing and command Escape handling pause while ATTEDIT drafts are open; the editor
retains field editing, Apply, Cancel and popup handling. The editor is explicitly
ordered above its shield: a first candidate let an outside click cover the editor's
controls, reproduced by extending the regression to click Cancel afterward.
Tab keeps native focus navigation while an egui modal is active instead of becoming
command-line completion text. This correction applies to other egui modals too;
ordinary command-line Tab conversion remains unchanged.

Files: ui-egui/src/attributedlg.rs and attributedlg/tests.rs, cmdline.rs, menus.rs;
this ledger and UI/inventory records. Five new checks cover outside selection and
subsequent Cancel, Cancel over canvas, Delete/F7/command typing and resumption,
an active LINE command, and modal versus ordinary Tab processing. Existing
value/format tests remain.

Dependencies/reversal: builds on CC-005's attribute editor; the Text Options Cancel
regression uses CC-045. It does not depend on CC-046's export fix. Revert this arc
to remove its modal shield, ATTEDIT keyboard guards and shared modal Tab correction;
other editor/value/format behavior stays. Out-of-order reverts of CC-045 require
retaining this wrapper and adapting its Text Options test. No data migration or
saved drawing changes. Programmatic commands remain available and existing draft
staleness handling is retained. Detached native ribbon-window modality has not
been verified and is not claimed.

Validation: corrected source passed 1,086 focused workspace library/binary and
attribute integration tests, none failed or ignored, and the Windows build. All six
cargo xtask ci gates passed (fmt, Clippy, tests, assets, layers and wasm): 2,369
passed, none failed or ignored. Windows Computer Use confirmed outside clicks keep
the editor active and selection unchanged, Cancel remains usable afterward, Text
Options/dropdowns remain usable, and Escape dismisses the dropdown alone. No test
attribute value was changed. Every GUI build quit normally before another launched;
all test documents were clean. The same title-bar drag gesture did not move either
the prior CC-046 build or this build, so native dragging is unverified rather than
claimed fixed or proved regressed. No private drawing, Autodesk output or asset
is committed. Inverse-patch verification is recorded with the implementing commit.

## CC-048 — per-reference attribute Properties tab

Reference: AutoCAD 2027 Enhanced Attribute Editor Properties was checked on a
separate CADCraft-generated Equipment DWG with an AttributeTest cyan layer and
loaded original-library linetypes. DEVICE_TYPE Layer AttributeTest previewed only
the title cyan; selecting DASHED and Apply retained those settings. Switching to
DEVICE_ID showed Layer 0, Linetype/Color/Lineweight ByLayer. Cancel kept the applied
result; one Undo restored the title white. Earlier checks covered Red and 0.35 mm
on one attribute. No AutoCAD-produced drawing or screenshot is committed.

Change: a Properties tab exposes loaded layers/linetypes, named/indexed/RGB color
entry with basic color choices, and standard metric lineweights plus ByLayer and
ByBlock. Initial values read each attribute's overrides over its matched definition;
Layer 0 remains 0. Only explicitly edited fields become ATTEDIT properties, preserving
inherited defaults, unrelated fields and external changes on no-op Apply. Apply,
OK and Cancel retain their existing draft semantics; all edited tabs use one command.

ATTEDIT accepts optional per-tag properties (layer/color/linetype/lineweight). All
property and text-format edits validate before any value mutation. Names canonicalize
to loaded tables; unknown/ambiguous/duplicate tags and unknown fields are rejected.
Color/tag/name inputs are bounded; invalid/nonfinite/nonstandard lineweights are
rejected instead of clamped. Constants and locked/out-of-working-set references
are protected. Existing overrides, placement and multiline text remain intact.
Programmatic Default lineweight is supported for existing legacy data, but was
not observed as a new AutoCAD editor choice and is not offered in the dropdown.

Files: engine/src/cmd/blocks.rs, blocks/attedit.rs, new attedit_properties.rs;
engine/tests/attribute_properties.rs; ui-egui/src/attributedlg.rs, new
attributedlg/properties.rs and attributedlg/tests.rs; this ledger and parity records.

Dependencies/reversal: the combined editor uses CC-005, CC-045's Text Options and
CC-047's input shield. Core property validation uses the upstream AttribProps model;
it does not require CC-046's baseline fix. Revert this complete arc to remove the
Properties tab and ATTEDIT properties parameters while retaining value/Text Options
editing and the input shield. Out-of-order removal of CC-045/047 needs context
resolution preserving this tab and its tests, or reverse CC-048 first. No file-format
migration: saved overrides remain readable by the existing model after code reversal.
A reversal does not undo edits already saved. Full Select Color dialog, live preview,
plot-style editing and text-effect/annotation controls remain separate gaps.

Focused validation passed 1,096 tests, none failed or ignored: six engine checks cover atomic combined edits, both saved formats, multiline contents, invalid inputs, constants/locks/ambiguous tags and standard lineweights/colors; four UI checks cover choices with real dropdown scrolling, Cancel, errors, inherited defaults and stale-draft protection. A final bound on unknown-field error text and a long-key case were added before the Windows/full-CI pass; those passes must validate the final source before publication.

Windows verification on the CADCraft-generated reference DWG: the Properties tab
selected AttributeTest, DASHED and 0.35 mm for DEVICE_TYPE, then Apply. The title
became cyan while DEVICE_ID remained unchanged; switching the target to DEVICE_ID
showed Layer 0 and ByLayer for its remaining properties. Cancel retained the applied
edit. Native Ctrl+Z removed all three overrides together and restored the white title;
Ctrl+Y restored the cyan title and all three overrides. Separate saved DXF and DWG
files reopened with the same title settings and untouched ID. The saved files and
readback log stay outside Git.

AutoCAD 2027 readback verified both a direct DWG save and a DWG saved after reopening
the DXF: both attributes open in Enhanced Attribute Editor. DEVICE_TYPE reads
AttributeTest, DASHED, Color ByLayer (cyan swatch) and 0.35 mm; DEVICE_ID reads
Layer 0 and ByLayer for the other three fields. Both values remain intact. Initial
unsuccessful text picks and a message after cancelling selection did not establish
lost attributes: picking the unchanged ID's P stroke and allowing the editor to
finish opening succeeded on both files. No AutoCAD file was saved.

Windows color verification: invalid color text leaves the editor open with an error
and the model unchanged. Choosing Red then Apply clears the error, stores ACI 1
only on DEVICE_TYPE and leaves its other overrides and DEVICE_ID unchanged. Cancel
retains the applied edit. Combined invalid value/format/property atomicity is
covered by the UI and engine tests. One Undo restores the cyan ByLayer title while
retaining the earlier layer/linetype/lineweight edits. All six CI gates passed:
2,379 tests passed, none failed or ignored; fmt, clippy, assets, layers and wasm
passed. The final-source Windows executable rebuilt successfully; reopening the
saved DWG, opening Properties and Cancel were rechecked in that executable.
Detached native ribbon mouse input during this editor remains a
confirmed separate gap: clicking Line starts a background command while the editor
stays open. CC-047 protects the root window; it does not cover that native child.

## CC-049 — floating ribbon input guard during attribute editing

Reference: AutoCAD 2027's detached Properties panel becomes gray during Enhanced
Attribute Editor. Clicking its color dropdown does not open a menu or change the
drawing; Cancel restores the enabled controls. Checked through Computer Use on an
own synthetic drawing, without saving an AutoCAD-produced file.

Before: CADCraft's detached Home / Draw panel could start LINE while Edit Attributes
remained open. The root-window input shield did not extend to another OS viewport.
Change: the shared native ribbon callback disables panel contents during ATTEDIT,
including whole-ribbon tools and Return to Ribbon. Pointer/text events in that child
are withheld so a popup opened earlier cannot bypass the disabled parent. Root
viewport popups are retained. Normal tools resume when the editor closes. This is an
application input guard, not an OS-wide modal window implementation; window moving
and resizing remain native-host behavior. Other dialogs are outside this change.

Files: ui-egui/src/ribbon/floating/native.rs and tests/native_ribbon_viewports.rs;
this ledger and UI parity/inventory records. No drawing or settings migration.

Dependencies/reversal: uses CC-028/031's native viewports and CC-047's ATTEDIT state
and root shield. Independent of CC-048's Properties tab. Revert the complete arc to
restore earlier native child tool input; attribute editing, formatting,
saved properties and normal ribbon geometry remain. Later changes that rely on
this shared callback guard must be adapted or reversed first. Reversal does not
modify saved drawings or ribbon placement.

Focused native-viewport suite: 16 passed, none failed or ignored. New regressions
cover both floating forms, blocked clicks, re-enabled LINE, an
already-open color popup and retention of a real root popup. The tool regression
failed before the fix. The first whole-ribbon test used its compact layout (which
has a Draw launcher rather than an inline Line button); it now uses a wide ribbon
to test the actual tool. The root-popup test keeps a real popup alive each frame,
rather than asserting on an expired unrendered popup.

Windows checks: a detached Draw panel and a wide whole-ribbon window both turn gray
during Edit Attributes; clicking Line leaves `running` empty and the editor open.
The root Properties layer dropdown remains usable. The final executable was also rechecked: after Cancel, detached Line
starts normally and can be cancelled without adding geometry. All prior GUI builds
were closed before this build started. The native close-button attempt retained
both windows and a responsive command API, but Computer Use returned `no monitor
found for window` for root capture and again after reselecting the root. Restarting
the clean test instance recovered capture. Consequently native close deferral is
not included in the final change: native close handling retains its prior behavior.
Normal whole-ribbon closing after Cancel was verified on Windows. The first candidate
passed all six CI gates (2,381 tests, none failed or ignored). After removing the
close-deferral addition, the final Windows executable rebuilt and passed blocked/
restored Line checks. Final-source CI passed all six gates: 2,381 tests passed,
none failed or ignored; fmt, clippy, assets, layers and wasm passed.

## CC-050 — attribute Select Color draft picker

Reference: AutoCAD 2027 Enhanced Attribute Editor > Properties > Select Color
opens a nested dialog, with Index Color, True Color and ByLayer/ByBlock choices.
The parent is disabled; cancelling the color dialog returns to the attribute
editor without committing a color. The RGB field was edited and cancelled during
Computer Use on our synthetic block. Color Books contents were not inspected.

Change: an original Select Color picker for the selected attribute's draft,
reusing CADCraft's generated ACI palette and RGB spectrum/components. A bounded
color-value field accepts existing names/indexes/hex/RGB syntax; invalid values
cannot be accepted. Use Color accepts to the draft only. Cancel Color or Escape
returns to the still-open parent, whose Cancel discards unapplied edits. Apply/OK
continues through the existing atomic ATTEDIT command and Undo. The parent is
disabled while picking; ATTEDIT remains active so root/native ribbon input guards
continue to apply. Each opening resets palette temporary state to the target draft.

The shared palette gets a caller-owned draft mode; its existing layer/ribbon popup
mode retains Apply true color. RGB component labels now label their number controls
for accessibility and interaction tests. No engine/file-format/settings migration.

Files: ui-egui/src/attributedlg.rs, attributedlg/{color_picker,properties,tests}.rs,
ui-egui/src/layers.rs, this ledger and UI parity/inventory records.
Dependencies/reversal: depends on CC-048's Properties drafts and uses CC-047's
editor shield. CC-049 guards native windows independently. Revert this whole arc
to restore the basic color choices/text field; saved attribute colors remain valid.
Later callers of color_palette_draft must be adapted first. No other callers use
that new entry point. Existing palette behavior must remain covered when reverting
its shared implementation. No Autodesk assets, color books or produced files copied.

Remaining differences: this picker uses CADCraft's original spectrum and explicit
Use Color/Cancel Color labels. It does not implement color books or the
reference HSL model controls; ByLayer/ByBlock are shown by logical name rather than
claiming an unverified resolved preview. Live drawing preview is a separate gap.
Targeted tests, full CI and final-source Windows checks are recorded below.

Windows verification: final-source executable built in 2m35s; prior GUI builds
closed after confirming all test documents clean. Index palette/True Color fit
without clipping. Escape returned only to the parent; RGB 12,34,56 updated the
spectrum/component readouts. Use Color changed only the draft (raw attribute
properties still inherited color); parent Apply changed only DEVICE_TYPE.
Cancel retained that applied value. GUI Undo restored its original ByLayer cyan
and GUI Redo restored RGB. Saved to a separate synthetic DWG and reopened from
disk: RGB, AttributeTest/DASHED/0.35mm remained and DEVICE_ID had no overrides.
AutoCAD 2027 opened that CADCraft-generated DWG; native Enhanced Attribute Editor
read Color 12,34,56 plus the same layer/linetype/lineweight and both values.
Cancelled without saving an AutoCAD file. Native foreign-file provenance notice
was acknowledged for this one test output without changing the persistent setting.
A remaining visual difference was observed: disabling the parent window fades its
frame/translucency as well as its controls; opaque parent-frame styling is a later
polish item. Focused attribute-editor suite: 17 passed (three new picker tests).
Clippy passed after collapsing three conditionals. All six CI gates passed: 2,384 tests passed, none failed or ignored; fmt, clippy, assets, layers and wasm passed.

## CC-051 — opaque disabled attribute-editor parent

Trigger/reference: while CADCraft's Select Color dialog is open, the disabled
Edit Attributes window fades its frame as well as its controls, allowing unrelated
ribbon labels to show through. AutoCAD 2027's disabled parent remains opaque.
Both appearances were inspected through Computer Use with synthetic drawings.

Change: suppress egui's disabled-area opacity only for rendering that parent,
then restore the exact original global style before rendering the nested picker
or other controls/viewports. The nested input shield supplies the dimming. Parent
controls, title-bar interaction and existing modal behavior remain disabled.
The color draft, Apply/Cancel/Undo, saved colors and ribbon guards are unchanged.

Files: ui-egui/src/attributedlg.rs, attributedlg/tests.rs and this ledger/UI parity/
GUI inventory. Existing picker interactions now also assert restoration of the
normal disabled alpha; the RGB reopen case uses a non-default alpha of 0.37.
No drawing/configuration migration, assets or external dependencies.

Dependencies/reversal: requires CC-050's nested picker. Revert this arc to restore
the prior parent translucency without removing picker functionality or saved data.
Later parent-style changes need conflict resolution if this arc is reverted.
CC-047/049 input guards and CC-048 property commands remain independently useful.
Windows visual confirmation and the full quality gates passed, as recorded below.
Windows final-source build passed in 3m02s. The disabled parent stays opaque with both True Color and Index Color tabs; underlying ribbon labels no longer show through. Its close button remains blocked. Cancel Color returns to enabled parent controls, parent Cancel closes the editor, native ribbon appearance resumes, and both test documents remain clean. All six CI gates passed: 2,384 tests passed, none failed or ignored; fmt, clippy, assets, layers and wasm passed.
