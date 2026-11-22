// Rack for three display boards, held by their short edge. Units: mm.
// Axes: x = width (0 = side against the stand, slots open to +x), y = depth (0 = front), z = height (0 = bottom).
// Every size that depends on a real part or on the printer is a named number here, so one change fixes the model.
// Command line: openscad -D print=true -o stl/rack.stl rack.scad
//               openscad -D print=true -D test_piece=true -o stl/rack_test.stl rack.scad

print = false;        // true = rotate so the left face lies on the bed and the slots open upward
test_piece = false;   // true = only the middle board's slot, for a quick fit test

// --- printer-dependent ---
play = 0.2;           // extra slot width; tune with the test piece
pcb_t = 1.6;
slot_w = pcb_t + play;

// --- bar ---
bar_w = 20;
bar_d = 15;
bar_h = 120;
slot_depth = 8;       // reaches the corner holes
end_gap = 10;         // top and bottom margin, also the height of the stand tabs
mid_gap = 7.5;

// --- boards, top to bottom: 6-digit, 4-digit, LCD (estimates, see measurements.md) ---
board_short = [25, 24, 36];
hole_edge = 2.75;               // corner hole centre from the clamped edge
hole_span = [20, 20, 31];       // distance between the two corner holes of a board
win = [[7, 18], [6.5, 17.5], [22, 30.5]];   // plug window z range, measured from the board's bottom edge
win_face = ["front", "front", "back"];      // wall that gets the window
win_reach = 6;                  // window length past the clamped edge, covers the plug body

// --- screws ---
screw_clear_d = 2.4;            // M2 clearance in the front wall
screw_pilot_d = 1.7;            // M2 self-tapping pilot in the back wall
screw_pilot_len = 5;
min_wall = 1.5;

// --- stand tab (placeholders until the stand is measured) ---
tab_t = 3;
tab_len = 12;
tab_hole_d = 3.4;

floor_x = bar_w - slot_depth;
slot_y = (bar_d - slot_w) / 2;
screw_x = floor_x + hole_edge;

// Top edge of board i; recursion keeps it a pure function of the parameters.
function top_of(i) = i == 0 ? bar_h - end_gap : top_of(i - 1) - board_short[i - 1] - mid_gap;
function bottom_of(i) = top_of(i) - board_short[i];
function hole_zs(i) = [(board_short[i] - hole_span[i]) / 2, (board_short[i] + hole_span[i]) / 2];
// Distance from z to the interval [a, b]; 0 if inside.
function dist_to(z, a, b) = z < a ? a - z : z > b ? z - b : 0;

assert(bottom_of(2) >= end_gap - 0.01, "boards do not fit: last board enters the bottom tab area");
assert(slot_depth >= hole_edge + screw_pilot_d / 2 + min_wall, "slot too shallow for the corner holes");
assert(bar_d - slot_y - slot_w >= screw_pilot_len + min_wall, "back wall too thin for the screw pilots");
assert(len([for (i = [0:2], z = hole_zs(i)) if (dist_to(z, win[i][0], win[i][1]) < screw_pilot_d / 2 + min_wall) 1]) == 0,
       "a plug window comes too close to a screw hole");

module slot(i) {
  translate([floor_x, slot_y, bottom_of(i)]) cube([slot_depth + 1, slot_w, board_short[i]]);
}

// Window in the front or back wall, open to the left face, so the plug and cable leave sideways.
module window(i) {
  y0 = win_face[i] == "front" ? -1 : slot_y + slot_w - 0.01;
  y1 = win_face[i] == "front" ? slot_y + 0.01 : bar_d + 1;
  translate([-1, y0, bottom_of(i) + win[i][0]])
    cube([floor_x + win_reach + 1, y1 - y0, win[i][1] - win[i][0]]);
}

// Both screws go from the front, through the board hole, into the back wall.
module screws(i) {
  for (z = hole_zs(i)) translate([screw_x, 0, bottom_of(i) + z]) rotate([-90, 0, 0]) {
    translate([0, 0, -1]) cylinder(d = screw_clear_d, h = slot_y + 1.01, $fn = 24);
    translate([0, 0, slot_y + slot_w]) cylinder(d = screw_pilot_d, h = screw_pilot_len, $fn = 24);
  }
}

module tabs() {
  for (z0 = [0, bar_h - end_gap])
    translate([bar_w / 2, bar_d - tab_t, z0]) cube([bar_w / 2 + tab_len, tab_t, end_gap]);
}

module tab_holes() {
  for (z0 = [0, bar_h - end_gap])
    translate([bar_w + tab_len / 2, bar_d - tab_t - 1, z0 + end_gap / 2])
      rotate([-90, 0, 0]) cylinder(d = tab_hole_d, h = tab_t + 2, $fn = 24);
}

module rack() {
  difference() {
    union() { cube([bar_w, bar_d, bar_h]); tabs(); }
    for (i = [0:2]) { slot(i); window(i); screws(i); }
    tab_holes();
  }
}

module part() {
  if (test_piece)
    intersection() {
      rack();
      translate([-1, -1, bottom_of(1) - 4]) cube([bar_w + tab_len + 2, bar_d + 2, board_short[1] + 8]);
    }
  else rack();
}

if (print) translate([bar_h, 0, 0]) rotate([0, -90, 0]) part();
else part();
