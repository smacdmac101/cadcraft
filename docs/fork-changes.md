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
