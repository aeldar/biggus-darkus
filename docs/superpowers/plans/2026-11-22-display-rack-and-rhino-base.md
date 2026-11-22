# Display Rack and Rhino Base Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Parametric OpenSCAD models, and printed parts, for a three-board display rack and a Rhino-shaped two-part case for the 70 x 50 mm perfboard.

**Architecture:** One OpenSCAD file per part in `3d-print/`, with every size as a named number at the top. "Tests" are `assert()` lines inside the files plus a check that `openscad` exports a valid STL with the expected bounding box. Physical fit is checked with short test prints.

**Tech Stack:** OpenSCAD (run with `nix shell nixpkgs#openscad -c openscad ...`, it is not installed), STL output, PLA on a library printer.

**Spec:** `docs/superpowers/specs/2026-11-22-display-rack-and-rhino-base-design.md`

## Global Constraints

- Rack bar about 120 x 20 x 15 mm (height x width x depth); slot 1.8 mm wide (1.6 mm PCB + 0.2 mm play), about 8 mm deep.
- Board order top to bottom: 6-digit TM1637, 4-digit TM1637, 1602 LCD; short sides about 25, 24, 36 mm (to be measured).
- Two M2 self-tapping screws per board through the corner holes, front into back wall.
- Plug windows open to the left: front wall for the TM1637 plugs, back wall for the LCD plug.
- Two-hole back tab at the top and bottom ends for the aluminium stand.
- Base: two halves joined by screws; perfboard 70 x 50 mm, M2 holes (2.2 mm) 1.98 mm from left/right edges and 2.14 mm from top/bottom edges; shell about 90 x 65 x 40 mm.
- No supports needed to print; PLA; each part fits a 20 cm bed.
- Ask the user before every `git add` and `git commit`; commits are unsigned (`--no-gpg-sign`).

## Review Focus

- Measured board sizes differ from the estimates: changing one number at the top must change the model (no hard-coded copies).
- Printer is looser or tighter than 0.2 mm play: play is one named number.
- Plug window collides with a board screw or a slot: assert a minimum 1.5 mm wall between them.
- Base lid cutouts miss the jack or USB port: check against the hole positions in `pcb/wiring.md`.
- Library rejects the file (type, size): confirm rules before the final print (Task 5).

---

## File Structure

- `3d-print/rack.scad`: the rack and its tab.
- `3d-print/base.scad`: both halves of the Rhino case (module per half, plus an assembled preview).
- `3d-print/README.md`: how to render and print (updated at the end).
- `3d-print/stl/`: exported STL files (committed, small).

### Task 1: Measure and record the real sizes

**Files:**
- Create: `3d-print/measurements.md`

**Interfaces:**
- Produces: the numbers in `measurements.md` that `rack.scad` and `base.scad` use as their named parameters.

- [ ] **Step 1: With calipers, record per board** (6-digit, 4-digit, LCD): short side, hole diameter, hole distance from the clamped short edge, hole spacing along that edge, plug position and height. Also the stand's hole spacing and screw size.
- [ ] **Step 2: Write them in `measurements.md` as a table**, one row per number, marked "measured" or "estimate".
- [ ] **Step 3: Ask the user before committing.**

### Task 2: Rack model

**Files:**
- Create: `3d-print/rack.scad`

**Interfaces:**
- Produces: `module rack()` (the whole bar with slots, windows, screw holes, tab) and the parameters `play`, `pcb_t`, `slot_depth`, `board_short[3]`, `gap`, `tab_hole_d`, `tab_hole_spacing`.

- [ ] **Step 1: Add the parameters at the top and `assert` lines:** total of `board_short` plus gaps is at most 120; `slot_depth` is at least hole distance plus screw radius plus 1.5 mm; windows leave at least 1.5 mm wall to the screw holes.
- [ ] **Step 2: Run** `nix shell nixpkgs#openscad -c openscad -o /dev/null 3d-print/rack.scad`. Expected: no assert failure.
- [ ] **Step 3: Implement `module rack()`:** bar, three vertical slots open to the right, two M2 holes per board, windows on the left (front wall for boards 1 and 2, back wall for board 3), back tab at top and bottom with one hole each. Slot openings face up for printing.
- [ ] **Step 4: Export** `nix shell nixpkgs#openscad -c openscad -o 3d-print/stl/rack.stl 3d-print/rack.scad` and check the bounding box is about 120 x 20 x 15 (tab included in the first number).
- [ ] **Step 5: Ask the user, then commit.**

### Task 3: Rack test print

**Files:**
- Modify: `3d-print/rack.scad` (only the `play` and hole numbers)

- [ ] **Step 1: Export a short version** (one slot, 30 mm) with a `test_piece = true` parameter, print it.
- [ ] **Step 2: Check:** a board slides in by hand without force, the plug fits the window, the screw bites. Change `play` in steps of 0.1 mm until it does.
- [ ] **Step 3: Print the full rack, mount all three boards and the cables on the stand.** Expected: nothing touches, the boards do not wobble.
- [ ] **Step 4: Ask the user, then commit the final numbers.**

### Task 4: Rhino base

**Files:**
- Create: `3d-print/base.scad`

**Interfaces:**
- Produces: `module base_bottom()`, `module base_top()`, `module base_assembled()` (preview only), with parameters `board_w = 70`, `board_h = 50`, `hole_dx = 1.98`, `hole_dy = 2.14`, `wall`, `wheel_d`.

- [ ] **Step 1: Add the parameters and `assert` lines:** shell inner space is at least the board size plus 2 mm each side; standoff height clears the solder side by at least 3 mm; the shell fits 200 x 200 mm.
- [ ] **Step 2: Implement `base_bottom()`:** floor, four M2 standoffs at the board holes, joining screw posts. Implement `base_top()`: the Rhino body with six shallow raised wheels and plates, no overhangs; cutouts for the barrel jack, the Nano USB, and cable slots for J4 to J8 (positions from `pcb/wiring.md`).
- [ ] **Step 3: Run** `nix shell nixpkgs#openscad -c openscad -o /dev/null 3d-print/base.scad`. Expected: no assert failure.
- [ ] **Step 4: Export** both halves to `3d-print/stl/base_bottom.stl` and `3d-print/stl/base_top.stl`; check the sizes against the spec (about 90 x 65 x 40 together).
- [ ] **Step 5: Ask the user, then commit.**

### Task 5: Library rules and final prints

**Files:**
- Modify: `3d-print/README.md`

- [ ] **Step 1: Look up the Oslo library makerspace rules** (file type, bed size, materials, booking, cost) on the library's own site, and add a short "Printing" section to `3d-print/README.md`.
- [ ] **Step 2: Print the bottom half first as a fit test with the real perfboard.** Check the board holes line up and the jack and USB cutouts match.
- [ ] **Step 3: Print the top half and join the halves with screws.** Expected: it closes without force, cables leave through the slots.
- [ ] **Step 4: Ask the user, then commit the README and the final STL files.**

---

## Self-review

- **Spec coverage:** rack layout, windows, screws, tab (Tasks 2 to 3); base shape, standoffs, cutouts (Task 4); OpenSCAD with named numbers and `3d-print/` folder (all tasks); open items (measurements, stand holes, library rules, Rhino proportions) are Tasks 1 and 5 and the Rhino look in Task 4.
- **Consistency:** parameter names are the same where they repeat (`play`, `pcb_t`, `slot_depth`, `board_short`).
- **Proportion:** the plan is not much longer than the spec; the code bodies are left to the builder.
