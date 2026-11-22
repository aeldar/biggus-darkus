# 3D-printed parts

Printed parts for the Biggus Darkus device. Status: first models done with **estimated** sizes, nothing printed yet.

| Part | File | What it is |
|---|---|---|
| Rack | [`rack.scad`](rack.scad) | Light bar (120 x 15 x 20 mm, plus a 12 mm tab at each end) that holds the 6-digit display, the 4-digit display and the 16x2 LCD by their short edges. Bolts to the aluminium stand on the 2020 profile. |
| Base | [`base.scad`](base.scad) | Two-part Rhino-shaped case (about 87 x 67 x 27 mm, 74 mm wide with wheels) for the 70 x 50 mm perfboard (see [`../pcb/wiring.md`](../pcb/wiring.md)). Hangs on the display cables. |

Sizes and what is still an estimate: [`measurements.md`](measurements.md).
Design and plan: [`../docs/superpowers/specs/2026-11-22-display-rack-and-rhino-base-design.md`](../docs/superpowers/specs/2026-11-22-display-rack-and-rhino-base-design.md), [`../docs/superpowers/plans/2026-11-22-display-rack-and-rhino-base.md`](../docs/superpowers/plans/2026-11-22-display-rack-and-rhino-base.md).

## Export

```
openscad -D print=true -o stl/rack.stl rack.scad
openscad -D print=true -D test_piece=true -o stl/rack_test.stl rack.scad
openscad -D 'part="bottom"' -o stl/base_bottom.stl base.scad
openscad -D 'part="top"' -o stl/base_top.stl base.scad
```

The files stop with an error if a change breaks a rule (boards do not fit, a plug window touches a screw hole, and so on).

## Print

- Material: PLA. No supports needed.
- Rack: `print=true` stands it on its left face, slots opening upward. Print the test piece first and tune `play` in `rack.scad`.
- Base: the lid (`top`) is exported face down. Grooves on its top face are shallow recesses.
- Library rules (file type, bed size, booking): not checked yet.

## Decisions made while modelling

- Base height is 27 mm, not the 40 mm estimated in the design: computed from the Nano on its sockets and the capacitor.
- The halves meet at 18 mm, above the wheels, so neither half has overhangs.
- The lid has grooves, not raised ridges, because it prints face down.
- Cable slots in the long walls also cut through the wheels.
- The USB port is about 21 mm inside the board edge, so its cutout is larger than the port to let the plug reach it.
- Everything about the displays (plug windows, hole positions) is an estimate; the LCD top screw and plug window are close, so check them with the test piece.
