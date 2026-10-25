# AutoCAD 2027 Windows GUI inventory and CADCraft work list

Updated 2026-10-25. CADCraft baseline: `27403b3`; this inventory accompanies
CC-035–CC-043. Reference: the installed AutoCAD 2027 Windows application, inspected
through Computer Use. This is a first-level GUI inventory and prioritized work list,
not an exhaustive test of every nested dialog or command, and not a parity percentage.

Evidence: **Live** means directly observed in this pass; **Prior** refers to the
recorded live checks in the fork ledger; **Source** means CADCraft implementation was
read. A visible AutoCAD command does not prove its result works. Uninspected surfaces
remain explicitly open. No Autodesk artwork, screenshots, CUI files, help text, or
private drawing data is included. Reference inspection changed UI tabs and temporarily
toggled coordinate visibility, then restored Home and the original visibility choice;
no drawing was saved. CADCraft checks use the disposable Bracket sample.

| ID | Surface | AutoCAD observation / evidence | CADCraft baseline / evidence | Adjustment or next check |
|---|---|---|---|---|
| GI-01 | Application menu | Application button visible; nested file/recovery pages not opened in this pass | File menu, recent files and recovery commands exist (Source) | Compare nested file workflows and recovery with synthetic files; open |
| GI-02 | Quick access | Compact New/Open/Save/Save As/Plot/Undo/Redo and Share area (Live) | File/undo groups visible; remaining commands behind overflow, group visibility only (Live/Source) | P2: individual pinning, command order and toolbar placement |
| GI-03 | Home | Draw, Modify, Annotation, Layers, Block, Properties, Groups, Utilities, Clipboard, View (Live) | CC-039–041 add inline layer, color and linetype selectors; Annotate style controls tracked below (Live/Source) | P1: style selectors, full linetype manager and visual color picker; audit missing Groups access |
| GI-04 | Insert | Block, Block Definition, Reference, Import, Data, Linking & Extraction, Location (Live) | Block, attributes, references and import commands exposed; fewer panels (Source) | P1: attribute/editor completeness; P2: data/extraction and underlay controls |
| GI-05 | Annotate | Text/style controls, Dimensions/style controls, Centerlines, Leaders, Tables, Markup, Annotation Scaling (Live) | CC-042/043 add current/selected/mixed dimension/text-style controls and manager access; leaders/tables retain command buttons (Source) | P1: leader/table style selectors and dimension/text preview galleries; P2: scaling and centerline grouping |
| GI-06 | Parametric | Separate Geometric, Dimensional and Manage panels (Live) | Combined Constraints panel via existing Tool Sets catalog (Source) | P2: regroup commands and expose parameters consistently; verify each command |
| GI-07 | View | Viewport tools, named views, model viewports, review/history, palettes and interface controls (Live) | Navigation/palette/layout commands available, smaller ribbon coverage (Source) | P2: named views, explicit interface toggles and supported viewport controls |
| GI-08 | Manage | Action recorder, customization, applications, standards, cleanup and performance (Live) | Management/inquiry commands and dialogs, no comparable full surface (Source) | P2: expose supported cleanup/alias tools; P3: recorder/standards work |
| GI-09 | Output | Plot, batch plot, preview, page setup, plotter management, export controls (Live) | Plot/preview/publish/page setup/export commands exist (Source) | P1: end-to-end paper/scale/device validation; P2: richer output grouping |
| GI-10 | Layout context | Layout tab present while a paper layout is active; Layout, Layout Viewports, view creation and standards groups (Live) | Opening Layout1 leaves Home tools, with no Layout tab (Live) | **CC-036:** add supported layout/page setup/viewport/space/plot panels; 3D view creation remains open |
| GI-11 | Add-ins | App Manager panel (Live) | No equivalent add-in manager inspected (Source) | P3: define supported extension architecture before UI placeholders |
| GI-12 | Collaborate | Share, Forma, Traces/markup and DWG Compare entry points (Live; no cloud actions invoked) | Some compare/share commands; no matching team-workflow surface (Source) | P3: local compare first; cloud features require separate design |
| GI-13 | Express Tools | Blocks/attributes, text, modify, layout, draw, dimension, tools and web panels (Live) | Commands distributed across menus; no Express Tools ribbon (Source) | P2: map supported equivalent commands; do not add nonworking copies |
| GI-14 | Featured Apps | Tab visible, contents not inventoried | No equivalent catalog (Source) | P3; uninspected, no functionality claim |
| GI-15 | File tabs | Start, active drawing/close, new drawing and document-list entry point (Live) | Start, drawing/close, new/switcher controls (Live/Source) | P2: compare preview/list/close-all and unsaved-work behavior |
| GI-16 | Layout tabs | Model, ordered paper tabs, add/list controls (Live) | Model/Layout1/Layout2/add/list present (Live) | P2: in-place rename and Move/Copy interactions; use synthetic layouts |
| GI-17 | Status customization | Checked visibility menu; Coordinates toggled on/off, readout appears/disappears, menu stays open (Live) | Fixed set of toggles, no individual visibility menu (Live/Source) | **CC-037:** visibility menu, persistence and recovery, separate from drafting settings |
| GI-18 | Coordinate/status readout | Coordinates is an optional visible item (Live) | Coordinate readout shares bounded transient-message slot (Source) | CC-037 hides coordinates without suppressing important status messages |
| GI-19 | Model/paper and scales | Paper-space indicator and annotation/viewport controls visible (Live) | Paper/model toggle, viewport scale/lock and annotation scale exist (Live/Source) | CC-037 adds visibility choices; P1: matched drawing scale/lock checks |
| GI-20 | Workspace switching | Workspace entry visible in status customization (Live); preset menu not opened | Workspace button routes to `ui.noop` (Source) | P1: replace misleading placeholder with real saved workspace selection or disable it explicitly |
| GI-21 | Ribbon/palette placement | Whole-ribbon and individual floating/anchoring behavior checked in prior pass | Native ribbon windows, anchors/auto-hide and edge snapping (Prior, CC-025–034) | P2: continuous drag transfer, precise cancellation/locks, browser/native-platform checks |
| GI-22 | Side palettes | Properties/Tool Palettes/Blocks launchers observed on View (Live); palette bodies not refreshed this pass | Layers/Properties visible, shared docking/auto-hide; floating side palettes stay in app (Live/Source) | P1/P2: native side-palette windows and consistent width/focus/return behavior |
| GI-23 | Block attributes | Edit Attributes/Manage Attributes launchers observed (Live); value editor checked previously | Attribute values/editor implemented in earlier arcs (Prior/Source) | P1: formatting/alignment/style/multiline settings with synthetic round trips |
| GI-24 | Layers | Home layer combo and layer actions visible; current/selected/mixed-layer behavior and CHPROP Undo checked on two synthetic lines (Live) | CC-039 adds inline Home selector, layer-state actions and selection-aware edits in docked/detached panels | CC-039 verification in fork ledger; CC-040/041 cover color/linetype semantics; style selectors and full picker/manager remain P1 |
| GI-25 | Command line | Floating entry, close/customize/history/recent controls (Live) | Entry/history/recent/line-count controls exist (Source) | P2: compare completion settings, transparency, docking and focus while commands run |
| GI-26 | Drafting aids | Ortho/polar/isometric/tracking/osnap/dynamic input entries in status menu (Live) | Existing transparent toggle commands (Source/tests) | CC-037 visibility must not change enabled state or cancel/pick points in LINE |
| GI-27 | Canvas navigation | Paper boundary, viewport labels/navigation bar visible (Live) | Paper boundary, viewport labels, ViewCube/UCS controls (Live/Source) | P1: matched-file text/font/viewport fidelity; P2: navigation-menu completeness |
| GI-28 | Contextual editors | Layout observed now; Hatch/Table/Block Editor assessed in prior pass | Selection/editor contextual tabs already exist (Prior/Source) | CC-036 supports Layout alongside selection tabs; P2: text/dimension/xref contexts |
| GI-29 | Start page/options | Start tab and settings entry points visible; pages/dialogs not inventoried here | Start/options implemented (Source) | P2: dedicated dialog pass; no claim all settings match |
| GI-30 | Split buttons/panel expansion | Large tools, smaller tools, dropdown arrows and panel launchers visible (Live) | Primary/secondary tools, More/flyouts and launchers exist (Source) | P2: remembered split-button choice, expansion pinning and consistent sizing |
| GI-31 | Appearance/accessibility | Dense ribbon rows, clear panel separators and active/context accents (Live) | Larger original icons, different label density/fonts, compact/wide floating layouts (Live/Source) | P2: spacing, clipping and contrast at narrow widths/HiDPI; keep original assets |

## Order of work

1. CC-036 and CC-037 close two directly reproduced GUI gaps with existing engine commands.
2. Inline layer/property/style selectors, attribute-format completeness and matched-file
   visual/plot checks have the highest drafting value. Work from synthetic drawings first;
   opening different files in two applications is not a fidelity comparison.
3. Replace the workspace no-op, then expand native palette consistency, quick-access
   customization, layout management and command-line controls.
4. Broader 3D, standards, add-ins, collaboration and sheet-set workflows need separate
   feature work; a copied tab or a decorative button would not provide those capabilities.

## Acceptance for this pass

- Layout appears in paper layouts, survives entering a layout viewport, coexists with
  selection contexts, and disappears in Model/Start/Block Editor or when contexts are disabled.
- Choosing a regular tab remains possible. Context choice is transient and does not corrupt
  the saved regular tab. Supported tools dispatch through the existing command registry.
- Status customization is reachable from the rightmost button, a status shortcut menu and
  Panels. Choices persist and can be reset even after all optional controls are hidden.
- Visibility does not toggle Grid/Snap/etc., dirty a drawing, cancel an active command or
  feed a point through the popup. Coordinates can hide while error/status text remains.
- Run interaction/persistence/native regressions, inspect the Windows build, run all quality
  gates and record actual results in the fork ledger before committing/pushing.
