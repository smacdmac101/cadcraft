# UI parity

> **Last reviewed:** 2026-10-10 · **Last updated:** 2026-10-25 · **Change:** minor (selection-aware ribbon styles, attribute Text Options and value-edit export alignment) · **Target:** Autodesk AutoCAD 2027 for Mac (26.0), with the focused Windows pass below

How CADCraft's interaction compares with AutoCAD's: point entry, object snaps, tracking, dynamic
input, grips, selection, the command line, chrome and navigation. A drafter's speed in AutoCAD
comes from these, so "the command exists" isn't enough: it has to accept the same input, snap to
the same points and respond to the same keys. Work items: [gaps.md](gaps.md#uiux).

**Headline (estimated): UI/UX fidelity ≈ 48%.** The command line, point syntax, the common
running snaps, polar/ortho, object snap tracking and the palettes are real, and so are the
Extension, Parallel and Apparent Intersection snaps (#379); dynamic input is a read-only
tooltip; FROM, M2P, TK and point filters don't exist; hot-grip options are advertised but
ignored. Ribbon, palette and status shortcut menus now exist (CC-015–CC-038); broader drafting-context behavior still needs review. Remaining: **≈ 110–170 Opus 5.5 hours**.

The focused [AutoCAD 2027 Windows GUI inventory](autocad-2027-gui-inventory.md) records 31 surfaces, evidence levels and follow-ups. CC-036–CC-038 add paper-space Layout tools, per-item status customization and menu Escape protection. CC-039 adds a current/selected/mixed-layer ribbon selector with layer-state controls and independent reversal tracking; CC-040 adds current/selected/mixed color semantics, inherited swatches and indexed/RGB entry; CC-041 adds a loaded-linetype selector and library loading; the broader percentage below is not reassessed by these passes.

## How this was measured

- CADCraft: read from the source on 2026-10-10 (main at bd2412f): `crates/engine/src/{prompt,
  snap, select, grips, lib, units}.rs`, `cmd/*`, `crates/ui-egui/src/*`. Not run in this pass.
- AutoCAD: behaviour observed black-box in the running AutoCAD 2027 for Mac
  (`plan/autocad/01-observed-ui.md`, local only, with reference screenshots that are never
  committed), its menu tree (`xtask/data/menu-catalog.txt`) and Autodesk's documentation.
- Weights (how often a drafter hits each area): point entry 15%, object snaps 15%, tracking and
  polar 10%, dynamic input 10%, grips 10%, selection 10%, command line 10%, chrome and palettes
  10%, navigation 5%, keyboard and context menus 5%.

| Area | Weight | CADCraft | Weighted |
|---|---:|---:|---:|
| Point entry | 15% | 45% | 6.8 |
| Object snaps | 15% | 45% | 6.8 |
| Tracking, polar, ortho, grid/snap | 10% | 70% | 7.0 |
| Dynamic input | 10% | 45% | 4.5 |
| Grips | 10% | 40% | 4.0 |
| Selection | 10% | 40% | 4.0 |
| Command line | 10% | 60% | 6.0 |
| Chrome and palettes | 10% | 45% | 4.5 |
| Navigation | 5% | 55% | 2.8 |
| Keyboard shortcuts, context menus | 5% | 25% | 1.3 |
| **Total** | 100% | | **≈ 48%** |

## Point entry (≈ 45%)

| Input | AutoCAD | CADCraft | Evidence |
|---|---|---|---|
| `x,y[,z]`, `@dx,dy`, `@d<a`, `d<a`, `#x,y`, bare `@` | yes | yes; `#` equals absolute because there is no UCS; Z parsed then discarded | `prompt.rs:147-180` |
| `*x,y` (WCS override) | yes | yes (`*x,y`, `*@dx,dy`, `@*dx,dy`); the same as the plain forms because there is no UCS | `prompt.rs` |
| Direct distance entry | yes, honours ortho, polar and tracking paths | yes, ortho; polar only through the UI's snapped cursor | `lib.rs:782-793` |
| Angle forms: `45d30'`, radians, grads, surveyor `N45dE` | yes | yes | `units.rs:141-197` |
| Architectural/engineering distances `5'6-1/2"` | yes | yes | `units.rs:93-140` |
| `FROM` (base point + offset) | yes | yes: "Base point:", "<Offset>:" with `@` measured from the base; nests with M2P and filters | `pointmod.rs` |
| `M2P` / `MTP` (midpoint between two points) | yes | yes | `pointmod.rs` |
| `TK` (tracking) | yes | **no** | |
| Point filters `.x`, `.y`, `.xy`, `.z` | yes | yes (`.x` … `.yz`, "(need YZ):"); Z is asked for and dropped (no 3D) | `pointmod.rs` |
| Angle override `<45` | yes | yes: `<a`, `<<a`, `<<<a`; a typed distance or a pick follows the angle, typed coordinates win | `pointmod.rs` |
| Object snap overrides (`END`, `MID`, `INT` … `NON`) | yes | yes, every mode and `NON`, with long names and `_`/`'` prefixes, for the next pick only; a typed point is snapped within the aperture; a command's own keyword wins (`_cen` always snaps); also from the snap menu | `pointmod.rs` |
| Typed calculator `'CAL` | yes | no | |

## Object snaps (≈ 45%)

| Mode | AutoCAD | CADCraft (`snap.rs:135-334`) |
|---|---|---|
| Endpoint | every curve, solid/3D vertices, dimension ends | lines, arcs, polyline segments, open ellipses and splines, ray base, solid/trace corners |
| Midpoint | every curve incl. ellipse/spline arcs, polyline segments | segments only; **none on circles, ellipses, splines** |
| Center | arcs, circles, ellipses, polyarcs | arcs, circles, ellipses |
| Geometric center | any closed polyline, spline | closed LWPOLYLINE only |
| Node | points, dimension definition points, text alignment points | POINT only |
| Quadrant | circles, arcs, ellipses | yes |
| Intersection | every pair, extended intersection | among up to 200 nearby primitives; ellipse/spline pairs approximate ([geometry-parity.md](geometry-parity.md)) |
| Extension | yes, with dashed extension path | yes: rest on a line or arc end, then the line runs on (a ray) and the arc round its circle; dotted path and "Extension: d < a°" (`snap/tracking.rs`); not from ellipse, spline or polyline-arc ends |
| Insertion | text, mtext, blocks, attributes, attdefs | TEXT, MTEXT, INSERT; no ATTDEF or MLEADER |
| Perpendicular | every curve, deferred | lines, arcs, circles, xlines; deferred for the first point (#168) |
| Tangent | circles, arcs, ellipses, splines, deferred | arcs and circles; deferred for the first point; **no ellipse or spline** |
| Nearest | every curve | yes |
| Apparent intersection | projected intersection | yes in 2D: where objects near the cursor would meet if extended; an object rested on sends its whole extension, so its crossing with another object snaps too (`snap.rs` `osnap_with`, `snap/tracking.rs`) |
| Parallel | yes, with parallel path | yes: rest on a line, then a path parallel to it runs through the base point (`snap/tracking.rs`) |
| Temporary overrides typed at a prompt (`END`, `MID`, `INT` …) | yes | **no**: the text arrives as plain input |
| Shift/Ctrl+right-click snap menu | yes | yes: From, Mid Between 2 Points, Point Filters, the snap modes, None, Osnap Settings; no Temporary track point yet (`context_menu.rs`) |
| Tab to cycle candidate snaps | yes | yes, through the closest snap of each running mode under the cursor (`pointmod::snap_candidates`) |
| AutoSnap marker, tooltip, magnet, aperture box | yes | marker per mode and name label; aperture from APERTURE; no magnet, no aperture box |
| OSNAPZ, 3D object snaps | yes | no |

## Tracking, polar, ortho, grid and snap (≈ 70%)

| Feature | AutoCAD | CADCraft |
|---|---|---|
| Ortho (F8) | yes, Shift overrides it while held | yes; holding Shift turns it on or off while held (`Session::snap_cursor`, `snap/tracking.rs`) |
| Polar tracking increment | yes | yes, measured from ANGBASE; dotted path and "Polar: d < a°" tooltip (`snap/tracking.rs`, `canvas.rs` `draw_tracking`) |
| Additional polar angles, relative to last segment | yes | additional angles yes (POLARADDANG with POLARMODE 4); **relative to the last segment no** |
| Polar distance snap (PolarSnap) | yes | **no** |
| **Object snap tracking** (acquire points, alignment paths, intersections of paths) | yes, F11 | yes (#379): resting on a snap point acquires it (`+`), resting again releases it, seven at most; horizontal/vertical paths, or every polar angle with POLARMODE 2; Shift to acquire with POLARMODE 8; crossings of two paths, a path and an object, and a path and a polar path snap; tooltip names each path; a typed distance (direct distance entry) runs along the path the cursor is on, measured from the point the path comes from. Engine: `Session::snap_cursor`, `snap/tracking.rs`; AUTOSNAP bits 8 and 16 |
| Grid (rectangular, major lines, adaptive, limits) | yes | rectangular and isometric (30°, 90° and 150° lines), major lines, adaptive display |
| Snap (rectangular, isometric, polar snap) | yes | rectangular and isometric (SNAPSTYL 1: the snap Y spacing along the isometric axes), from SNAPBASE turned by SNAPANG |
| Isometric drafting (ISODRAFT, isoplanes F5) | yes | yes (#556): ISODRAFT and the status-bar toggle set SNAPSTYL and the plane; ISOPLANE / F5 / Ctrl+E cycle Left, Top, Right (SNAPISOPAIR); ortho, direct distance entry and the crosshair follow the plane's axes; ELLIPSE Isocircle (radius or Diameter, also for elliptical arcs); Drafting Settings snap style radio. Engine: `snap/iso.rs`, `cmd/settings/iso.rs` |
| Drafting Settings dialog | yes | yes (`dialogs.rs:36-80`) |
| Hatch and Gradient / Hatch Edit dialogs | yes | yes (`hatchdlg.rs`): pattern swatches, gradient colours and shapes, origin, pick points or objects, associative, separate, island detection; Hatch Edit adds boundaries from points or closed objects and removes selected loops; opened by HATCH, GRADIENT (HPDLGMODE), HATCHEDIT, right-click or double-clicking a hatch; no Preview or Inherit Properties |

## Dynamic input (≈ 45%)

AutoCAD's heads-up input (F12) has editable pointer-input fields (x, y), dimensional input
fields (length and angle) with Tab between them and lock icons, relative-by-default second
points, dimensional input on grips, a prompt with clickable options and a down-arrow menu.
CADCraft has pointer input boxes since #60 (`crates/ui-egui/src/dyninput.rs`): value boxes
beside the cursor while a command asks for a point; second and later points default to polar
distance and angle (DYNPIFORMAT = 1 for Cartesian); `,` and `<` lock a value and move to the next
box; Tab types the separator; Enter sends a relative entry (DYNPICOORDS = 1 for absolute) and
fills an empty box from the cursor; a plain number stays direct distance entry. Missing:
dimensional input on grips (length/angle fields while stretching), lock icons, the prompt with a
down-arrow options menu (DYNPROMPT), and DYNDIM.

## Grips (≈ 40%)

| Feature | AutoCAD | CADCraft |
|---|---|---|
| Stretch grips per object type | yes | lines (midpoint moves), arcs (start/mid/end), ellipses (axes), polylines (vertices; segment midpoints move or reshape the bulge), dimensions (text and definition points), MLEADER, viewport, wipeout, spline (`grips.rs:79-270`) |
| Text/MText grips | position, width, column grips | move the whole object |
| Hot-grip modes with Space/Enter cycling | Stretch, Move, Rotate, Scale, Mirror | yes (`canvas.rs:58-67`) |
| Hot-grip options: Base point, Copy, Undo, Reference, eXit | yes | **advertised in the prompt but not parsed**; the engine's JSON grip commands accept `copy`, the UI never passes it |
| Several hot grips at once (Shift-click) | yes | **no** (one handle) |
| Multifunctional grip hover menus (Add Vertex, Remove Vertex, Convert to Arc, Lengthen …) | yes | **no** |
| Grip dimensional input (length/angle fields) | yes | no |
| Dynamic block grips | yes | no (no dynamic blocks) |
| Grip appearance (GRIPSIZE, colours, hover) | yes | GRIPSIZE, GRIPS and GRIPOBJLIMIT honoured (Options ▸ Selection); no grip colours |

## Selection (≈ 40%)

| Feature | AutoCAD | CADCraft |
|---|---|---|
| Pick, implied window (left→right) and crossing (right→left) | yes | yes, coloured and dashed rectangles (`lib.rs:576-589`, `canvas.rs:823-836`), filled as SELECTIONAREA, WINDOWAREACOLOR, CROSSINGAREACOLOR and SELECTIONAREAOPACITY say (Visual Effect Settings) |
| Keywords W, C, WP, CP, F, BOX, AU, SI, M, G (group), CL (class) | yes | yes except CLass: W, C, F, WP, CP (#226); BOX, AUto, SIngle, Multiple, Group, Add, Remove, Undo (#428); ALL, Last, Previous |
| Press-drag selection windows (PICKDRAG) | yes | clicks or drag (2, default), or two clicks only (0); Options ▸ Selection ▸ Select using two clicks only, saved in the profile; drag-only mode (1) is unavailable |
| Lasso | yes | **no** |
| Selection cycling (SELECTIONCYCLING) | yes | yes (#1143): SELECTIONCYCLING 2 lists the objects under a click (type, layer; the row under the pointer highlights its object), 1 the badge; Shift+Space steps the highlight through them (#1145); no Ctrl+W |
| Selection preview, hover highlight | yes | selected and hovered objects redrawn in the selection colour, fills half see-through (SELECTIONEFFECT 1; 0 dashed), selection effect color; HIGHLIGHT (not saved), SELECTIONPREVIEW, PREVIEWEFFECT, PREVIEWFILTER (Visual Effect Settings) |
| PICKADD, PICKFIRST, PICKAUTO | yes | PICKADD (Shift), PICKFIRST |
| Stability of outlined fill highlights | yes | closed contours keep the same start, direction and ordering on repaints and GPU rebuilds; dashed filled glyphs, hatch/solid regions, arrowheads and wipeout outlines no longer shift with pointer movement |
| QSELECT, SELECTSIMILAR, FILTER | yes | QSELECT dialog; SELECTSIMILAR with SELECTSIMILARMODE (#428, SEttings asks for the bit code; no dialog); no FILTER |
| Object isolate/hide (ISOLATEOBJECTS, HIDEOBJECTS) | yes | yes, temporary (OBJECTISOLATIONMODE 0; #428): Tools > Isolate, UNISOLATEOBJECTS; no status-bar button |
| Groups selectable as one (PICKSTYLE) | yes | yes (PICKSTYLE 1, #417), Object grouping on Options ▸ Selection; associative hatch selection (bit 2) is stored, not used |

## Command line (≈ 60%)

| Feature | AutoCAD | CADCraft |
|---|---|---|
| AutoComplete (commands, sysvars, mid-string, synonyms) | yes | command list with Tab to accept (`cmdline.rs:30,176,317`); no sysvar or mid-string search confirmed |
| Aliases | about 250 in `acad.pgp` | 144 aliases across the commands (counted from source) |
| History (F2), Up/Down recall | yes | yes |
| Transparent commands (`'zoom`, `'pan`, `'osnap`) | yes | yes (`lib.rs:496-503`) |
| Enter/Space repeats last command | yes | yes |
| Clickable keywords in the prompt | yes | yes (`cmdline.rs:253-267`) |
| UNDO with Mark, Back, BEgin, End, Auto, Control | yes | count only, whole-document snapshots capped at 2,000 (`lib.rs:477`, `edit.rs:56`) |
| SETVAR for every system variable | ~1,000 sysvars | about 60 session/header variables by name (`sysvars.rs`) |
| Scripts (SCRIPT, `.scr`) | yes | script text via CLI/MCP/control; no `.scr` file command |
| Floating/docked command window, transparency, lines of history | yes | floating overlay, history lines setting |

## Chrome and palettes (≈ 45%)

| Feature | AutoCAD for Mac | CADCraft |
|---|---|---|
| Menu bar | 11 menus, 491 leaf items | same 11 menus; 245/491 items covered (233 by `cargo xtask parity`, which counts engine commands only, plus 12 UI-only items: see [parity-checklist.md](parity-checklist.md)); once a menu is open, pointing at another title opens that menu (#1567) |
| Tool Bar, File tabs, Start tab, layout tabs | yes | yes; compact quick access while the ribbon is shown, full classic toolbar with RIBBONCLOSE. Its overflow and saved group choices remain available. |
| Ribbon tabs and tool panels | yes | Home, Insert, Annotate, Parametric, View, Manage and Output tabs, plus an optional Modeling tab; named panels with primary/secondary tools and overflow. Move to Ribbon transfers Tool Sets into this presentation; Move to Tool Sets restores the palette. Both use the original command catalogs and shared tool visibility. Right-click tabs, tools or background for Show Tabs/Show Panels. Choices, titles and display modes are saved; compact tab/panel flyouts provide temporary access. Restore Ribbon and the Panels/Window menus recover hidden controls. Floating panels retain tools across tabs and return to their source slot. Contextual Hatch Editor, Table and Block Editor tabs activate from supported selections/editor state. Desktop floating ribbon panels use independent native windows with saved desktop positions and missing-monitor recovery; browser/unsupported hosts embed them. The entire ribbon can also be undocked into a resizable desktop window, with compact vertical tabs/panel flyouts, a wide horizontal layout and explicit Dock Ribbon/internal-grip double-click recovery. Its close hides the ribbon; panel close returns the panel. Whole-ribbon anchors, hover roll-up and Keep Open complement native floating auto-hide and app-edge docking controls (CC-032–034). Authored layouts and additional contexts remain open. |
| Palette docking and auto-hide | title-bar menu, docking, anchoring and hover expansion | Every side-panel title menu has Dock Left/Right, Float and Auto-hide. Floating palettes also have explicit dock header buttons. Auto-hide retains edge tabs when docked or title bars when floating; hover opens an overlay, leaving closes it, and Keep Open pins the palette. Tool Sets alone has Move to Ribbon in its right-click menu. Width, placement and auto-hide choices are saved. Floating palettes stay inside the CADCraft window; native palette windows and docking locks remain open work. |
| Tool Sets palette (Drafting, Modeling, … with flyouts) | yes | yes for 2D groups |
| Properties Inspector, Quick Properties | yes | yes (every control acts or is visibly unavailable, #182); a single block reference exposes editable attribute values and read-only Position X/Y/Z; coordinate editing is still missing |
| Block attribute editor | values, text options, attribute properties | Double-click an attributed block, or ATTEDIT/EATTEDIT with one selected, opens Attribute and Text Options tabs with Apply, OK and Cancel (CC-045). Single-line per-reference style, justification, height, width factor, rotation and oblique are editable; constants/locked references are read-only. Live preview, per-attribute effects/annotation/multiline controls and Properties tab remain missing; without a selected block the command uses prompts. |
| Layer Properties Manager (filters, states, VP overrides) | yes | yes |
| Blocks palette, Content | yes | Blocks window (click to insert, #28) |
| Status bar toggles | ~25 | most toggles; Annotation Visibility and Workspace do nothing |
| ViewCube | interactive 3D | clickable for 2D (Top/Home → extents); dragging does nothing (#7) |
| Navigation bar, SteeringWheels | yes | no |
| Workspaces | Drafting & Annotation, 3D Modeling | no |
| Options / Preferences dialog | many tabs | **no** (issue #23) |
| Customisation (CUI, aliases editor, tool palettes) | yes | Command Aliases dialog (ALIASEDIT, #1381); no CUI or tool palettes |
| Themes | Dark, Light | Dark, Light, System (#179) |
| Accessibility | VoiceOver basics | AccessKit through eframe; screen-reader coverage untested; magnifier issue #39; animation effects follow the OS setting (View ▸ Accessibility ▸ Animation Effects: System, On, Off) |

### Focused Windows comparison (2026-10-24)

The block-attribute pass used AutoCAD 2027 on Windows and an isolated copy of a user drawing.
It does not refresh the older global percentages above. The imported power-adapter title uses
middle/center justification; its value is visible in AutoCAD's Properties palette and attribute
editor. CADCraft now imports that justification and exposes the value in both corresponding
editing surfaces. Apply, Cancel and Undo were exercised in CADCraft's UI, and the reported crop
renders identically after CADCraft DXF and DWG save/reopen. This is local visual evidence, not
proof that the complete DWG round-trips without loss.

Next comparisons, in order:

1. CC-045 adds single-line per-reference style, justification, height, width factor, rotation
   and oblique options, with atomic validation, Undo/Redo and synthetic DXF round trips.
   Next: live preview, Backwards/Upside-down and annotation context; creating Align/Fit needs
   two points. This does not establish full enhanced-editor or private-file fidelity.
2. Multiline attributes: editing and wrapping, with Cancel leaving the original content intact.
   File-format support does not establish an equivalent interactive editor.
3. Multiple selected block references: common attribute values, mixed-value feedback and one-step
   Undo. The current Properties implementation deliberately handles one reference.
4. Per-attribute layer, colour and line properties, separate from the enclosing block's properties.

Public behavior reference: [AutoCAD 2027 Enhanced Attribute Editor](https://help.autodesk.com/cloudhelp/2027/ENU/AutoCAD-Core/files/GUID-414DC467-291B-4A52-A70D-1165CDEA8B39.htm).
No Autodesk assets or private drawing fixtures are included in the repository.

The toolbar visibility pass addresses the reported missing right-click menu. AutoCAD's
[ribbon display controls](https://help.autodesk.com/view/ACD/2027/ENU/?caas=caas%2Fdocumentation%2FCIV3D%2F2014%2FENU%2FfilesACD%2FGUID-627FCA86-38B1-404B-BB09-9FC3348B54F2-htm.html)
include checked panel visibility. CADCraft now offers the equivalent choice for its existing
top tool groups, including restoration after hiding all groups (CC-009). This update adds
the tabbed ribbon, per-tab panel choices, panel titles and an explicit minimized state
(CC-013–CC-017). The follow-up adds saved tabs, panel-title and panel-button minimize modes,
the display cycle, and temporary tab/panel flyouts (CC-021–CC-022). Detached panels now remain across tab switches, support title dragging and Return to Ribbon, and save
their positions (CC-025). Contextual Hatch Editor, Table and Block Editor tabs derive from the active
document and selection/editor state (CC-026); regular tab choice is retained and restored. Context
tabs use an accent treatment, can be disabled, and expose existing editing commands rather than
claiming all AutoCAD contextual controls. Workspace validation, recovery and reference translations
cover these additions (CC-027). Native ribbon windows can move outside the application and across monitors (CC-028–CC-030). Live Windows observation also confirmed individual Draw-panel dragging, persistence across tab changes, its return control, whole-ribbon Undock, compact vertical presentation and double-click grip docking. CC-031 adds whole-ribbon floating and saved geometry. CC-032–034 add whole-ribbon anchors, hover roll-up, pinning and native movement docking. Additional contexts, user-authored layouts and native side-palette windows remain gaps. The minimize choices were observed in AutoCAD 2027 on Windows and checked
against the public [display-state reference](https://help.autodesk.com/cloudhelp/2027/ENU/AutoCAD-LT/files/GUID-627FCA86-38B1-404B-BB09-9FC3348B54F2.htm).

Ribbon acceptance: select Annotate and start DIMLINEAR from its Dimensions panel; right-click a
command button and hide Draw; verify that hiding Block on Home does not hide it on Insert;
hide every tab and use Restore Ribbon; hide the whole ribbon and recover it from Panels or
RIBBON; restart and retain tab, panel, title and minimized choices. Compact windows scroll the
panel strip horizontally. Ribbon tools invoke the existing registry; they do not implement
different drawing behavior. [Fork change ledger](fork-changes.md) records all earlier and new changes.

The Tool Sets presentation is the compatible default for existing workspace files. Move to
Ribbon hides that duplicate view, retains customized hidden tools and selects the corresponding
Drafting/Modeling ribbon. Every original tool has a ribbon location; unimplemented Modeling
tools stay disabled. Hatch/Gradient/Boundary are in Draw's overflow. Moving back restores the
palette's saved placement. Redocking follows the observed AutoCAD 2027 title-bar approach and
the public [docking reference](https://help.autodesk.com/view/ACD/2027/ENU/?caas=caas%2Fdocumentation%2FACD%2F2014%2FENU%2Ffiles%2FGUID-893A3C0A-C735-49E4-878E-C8641DEAD618-htm.html).

## Navigation (≈ 55%)

| Feature | AutoCAD | CADCraft |
|---|---|---|
| ZOOM Extents, All, Window, Previous, In/Out, Center, Scale, Object | yes | yes, 50-entry history (`cmd/view.rs:107-149`) |
| ZOOM Realtime (drag), Dynamic | yes | **no** |
| Wheel zoom about cursor, ZOOMFACTOR, ZOOMWHEEL | yes | fixed factor (no ZOOMFACTOR) |
| Middle-drag pan, double-middle-click extents | yes | yes (`canvas.rs:538-583`) |
| Trackpad two-finger pan, pinch zoom (macOS) | yes | yes (#58) |
| Named views (VIEW) | yes | **no** (VIEW table not read or written) |
| Tiled model-space viewports (VPORTS) | yes | not confirmed: VPORTS maps to paper-space MVIEW |
| 3D orbit, ViewCube drag, SteeringWheels | yes | no (M11) |

## Keyboard and context menus (≈ 25%)

- Shortcut table: 28 entries (Cmd shortcuts, F1, F2, F3, F5, F7–F12) plus 23 command `.key()`
  shortcuts. AutoCAD for Mac's defaults are similar in number; F4 (3D osnap) and F6 (dynamic UCS)
  are missing.
- Delete erases the selection on macOS (#64); Escape cancels.
- Right-click on the canvas is Enter while a command runs; holding it (250 ms, AutoCAD's
  time-sensitive right-click) opens the command menu: Enter, Cancel, Recent Input, the prompt's
  options, Snap Overrides, Pan/Zoom. Otherwise it opens a shortcut menu (Repeat, Recent Commands,
  Clipboard ▸, Isolate ▸, the selection's editing commands, Draw Order ▸, Group, the selected
  object type's edit commands, Select Similar, Deselect All, Undo/Redo, Pan/Zoom; #325, #512).
  The command line, drawing tabs, layout tabs, Layers palette rows and the Properties palette
  have menus too (#512). Still missing: in-place layout tab rename, Move or Copy, Select All
  Layouts.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-25 | minor | CC-039: selection-aware ribbon layer control, compact Layers actions and explicit dependency/reversal records. |
| 2026-10-25 | minor | CC-035–CC-038: Windows GUI inventory, Layout context alongside selection contexts, persisted status visibility and menu Escape protection. |
| 2026-10-25 | minor | CC-032–CC-034: whole-ribbon anchors, hover auto-hide, pinning and native edge-drag docking; reference observed in AutoCAD 2027 on Windows. |
| 2026-10-25 | minor | CC-031: live AutoCAD confirmation of both floating forms; whole-ribbon desktop palette, adaptive layout, docking recovery, independent close and saved geometry. |
| 2026-10-25 | minor | CC-028–CC-030: independent native ribbon windows, desktop/DPI-aware saved positions and missing-monitor recovery; lifecycle, viewport diagnostics and native-callback regressions. |
| 2026-10-25 | minor | CC-025–CC-027: detachable ribbon panels with saved positions and return controls; transient Hatch Editor, Table and Block Editor contexts; recovery, strict validation, translations and interaction coverage. |
| 2026-10-24 | minor | CC-021–CC-024: ribbon display modes and temporary flyouts, saved display cycling, stable palette editor identities and strict panel-tab validation. |
| 2026-10-24 | major | CC-013–CC-017: tabbed ribbon, per-tab panel visibility, saved customization, minimization and recovery; compact quick-access toolbar and command/interaction coverage. |
| 2026-10-24 | minor | CC-018–CC-019: palette redocking controls, preserved width/placement, shared Tool Sets/ribbon presentation and tool visibility; compatible opt-in migration. |
| 2026-10-24 | minor | CC-020: consistent palette title menus, Tool Sets-only ribbon action, saved hover auto-hide for docked and floating palettes and Keep Open recovery. |
| 2026-10-13 | minor | Filled-region highlight boundaries are traversed in deterministic coordinate order, fixing dash-phase jumps on CPU hover/selection redraws and GPU highlight rebuilds; CPU dash-position, pixel and GPU mesh regressions |
| 2026-10-14 | minor | Selection cycling (#1143): the list and badge of SELECTIONCYCLING, Shift+Space through overlapping objects (#1145) |
| 2026-10-14 | minor | Hatch Edit dialog adds boundaries from picked points or closed objects and removes selected loops |
| 2026-10-14 | minor | Holding Shift overrides ortho while it is held (on when ORTHOMODE is off, off when it is on), as in AutoCAD |
| 2026-10-16 | minor | ALIASEDIT opens the Command Aliases dialog (#1381) |
| 2026-10-18 | minor | In-window menu bar: once a menu is open, pointing at another title closes it and opens that one (#1567) |
| 2026-10-13 | minor | Options ▸ Selection adds Select using two clicks only; PICKDRAG 0 disables drag-window creation/completion, 2 preserves clicks or drag, and the preference survives restart; UI pointer and profile regressions |
| 2026-10-12 | minor | Selected and hovered objects redrawn in the selection colour (text and arrowheads included); Options ▸ Selection laid out as AutoCAD's: Object grouping, Show grips, grip object limit, selection preview, Visual Effect Settings (selection area, preview filter) |
| 2026-10-12 | minor | Animation effects (dialog and menu fades, smoothed wheel scrolling and ⌘/Ctrl+wheel zoom) follow the operating system's setting; View ▸ Accessibility ▸ Animation Effects chooses System, On or Off |
| 2026-10-12 | minor | Hatch and Gradient / Hatch Edit dialogs (HATCH, GRADIENT, HATCHEDIT, double-click a hatch; HPDLGMODE) |
| 2026-10-12 | minor | Block references now show their stored insertion X/Y/Z in the Properties Inspector, including nonzero Z; fields are read-only |
| 2026-10-11 | minor | Isometric drafting (#556): isometric snap and grid, ISODRAFT, ISOPLANE (F5), ortho along isoplane axes, ELLIPSE Isocircle |
| 2026-10-11 | minor | Object snap tracking, Extension, Parallel, Apparent Intersection, additional polar angles (#379): tracking 30% → 70%, object snaps 40% → 45%; total 43% → 48% |
| 2026-10-10 | minor | Dynamic input 15% → 45% (pointer boxes, #60); press-drag windows (#63); total 40% → 43% |
| 2026-10-10 | major | First UI-parity audit: point entry, snaps, tracking, dynamic input, grips, selection, command line, chrome, navigation, keys |
| 2026-10-24 | minor | Single-reference attribute values in Properties; double-click and selected ATTEDIT/EATTEDIT open an undoable value editor with Apply/Cancel. Reference observed in AutoCAD 2027 on Windows; formatting tabs remain a gap. |
| 2026-10-24 | minor | Toolbar context menu and Panels menu expose checked tool-group visibility, Show All Tool Groups, side-panel visibility and interface bars. Tool-group choices persist in UI settings. |
| 2026-10-25 | minor | CC-040: selection-aware ribbon color selector, inherited swatches, exact current RGB, bounded floating popup and locked-selection protection. Full Select Color dialog remains a gap. |
| 2026-10-25 | minor | CC-041: current/selected/mixed ribbon linetypes with pattern previews and original-library loading; Quick Select retained in Properties More menu. |
| 2026-10-25 | minor | CC-042: selection-aware Annotate dimension-style selector with manager access, locked-object guard and detached-panel support. |
| 2026-10-25 | minor | CC-043: Annotate text-style selector, shared TEXT/MTEXT style assignment and annotative updates; current/selected/mixed state, manager access, native popup and reversal tracking. |
| 2026-10-25 | minor | CC-045: attribute Text Options for style/justification/height/width/rotation/oblique, atomic per-reference ATTEDIT formats, Apply/Cancel/Undo and locked-target protection; live preview, text effects, multiline options and Properties tab remain open. |
| 2026-10-25 | minor | CC-046: refresh ordinary justified attribute export baselines after real value edits through all ATTEDIT/-ATTEDIT routes; preserve no-op font offsets and Left/Aligned/Fit placement points. |
