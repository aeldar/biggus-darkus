# Display rack and Rhino base: design

Status: draft, waiting for approval. Nothing is built yet.

## Intent

Two 3D-printed parts for the Biggus Darkus device, printed on a library printer in Oslo (PLA):

1. A light **rack** that holds the three display boards and bolts to an aluminium extension on a vertical 2020 profile.
2. A **base** shaped like the Elite Dangerous Rhino (six-wheeled surface-mining buggy) that holds the 70 x 50 mm perfboard. It is not fixed anywhere; it hangs on the display cables.

## Goals

- Rack: as thin and light as possible; easy to print without supports.
- Rack: every board is held by its short edge, the rest of the board is free.
- Base: two halves joined by screws; the board is fixed inside with its four M2 corner holes.
- Everything fits a normal library printer bed (about 20 cm per side) and prints in PLA.

## Non-goals and anti-goals

- No mounting of the base to anything.
- No flat full-size panel behind the displays (the user wants a rack, not a stand).
- No parts that need supports or glue to work.

## Rack

Layout (user photo): a vertical bar, boards stacked top to bottom, all facing the viewer, left short edges lined up, boards stick out to the right.

| Item | Value | Source |
|---|---|---|
| Bar size | about 120 x 20 x 15 mm (height x width x depth) | user |
| Order, top to bottom | 6-digit TM1637, 4-digit TM1637, 1602 LCD | user photo |
| Short sides | about 25, 24 and 36 mm | estimate from photo, to be measured |
| PCB thickness | 1.6 mm, slot 1.8 mm (0.2 mm play) | user, my play value |
| Slot depth | about 8 mm, enough to reach the corner holes (about 2.5 to 3 mm from the edge) | my call |
| Board screws | two per board, through the corner holes, from the front into the back wall (M2 self-tapping) | my call |
| TM1637 plug | on the front face, about 3 mm from the edge, cable goes left: window in the front wall, open to the left | user |
| LCD plug | on the back face, about 8 mm behind the edge, near the top: window in the back wall, open to the left | user |
| Fixing to the aluminium stand | a thin flat tab on the back, sticking out to the right at the top and bottom ends, one hole each | user (2 holes) |

Approaches considered:

1. One solid bar with three slots, windows, screws through the board holes. **Chosen** (simplest, one part, no supports).
2. Two plates (grooved back, clamping front). Thinner, but two parts and the board holes must line up exactly. Kept as fallback if the windows make the walls too weak.
3. Snap clips. Too weak in PLA with cables pulling.

Print orientation: lying on its back, slot openings facing up.

## Base (Rhino)

| Item | Value | Source |
|---|---|---|
| Board | 70 x 50 mm perfboard, four M2 holes (2.2 mm) 1.98 mm from the left and right edges, 2.14 mm from the top and bottom edges | `pcb/wiring.md` |
| Shell | two halves (top and bottom), screws join them | user |
| Size | about 90 x 65 x 40 mm; the 470 uF capacitor and the barrel jack are the tallest parts | my estimate |
| Look | boxy armoured buggy with six wheels as shallow raised details, no overhangs | my call |
| Cutouts | barrel jack, Nano USB, cable slots for J4 to J8 on the top edge | my call, from `pcb/wiring.md` |
| Board fixing | four M2 standoffs in the bottom half | my call |

## Tools (my calls)

- OpenSCAD files with all sizes as named numbers at the top, so a test print can be corrected by changing one number. Output is STL, which printers usually accept.
- Folder: `3d-print/` in this repo (keeps it apart from `app`, `firmware`, `pcb`).

## Open items for the builder

- Exact short sides and hole positions of the three boards (measure with calipers).
- Hole spacing and screw size of the aluminium stand, to place the two tab holes.
- Library rules (file type, bed size, material, booking). To be checked on the library's site before printing.
- Rhino look: exact proportions are the builder's choice, from reference pictures.

## Plan (next step, after approval)

1. Measure the boards and the stand holes.
2. Model the rack, print a short test piece with all three slots, fix the play values.
3. Print the full rack, check fit and cables.
4. Model the Rhino base around the real board, test fit with a quick print of the bottom half.
5. Print the final base.
