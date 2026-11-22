// Rhino-style two-part case for the 70 x 50 mm perfboard (see ../pcb/wiring.md). Units: mm.
// Axes: x = length (the perfboard's 70 mm side), y = width, z = up. The board is centred on x = 0, y = 0.
// Command line: openscad -D part=\"bottom\" -o stl/base_bottom.stl base.scad   (part = "bottom" | "top" | "both")

part = "both";

// --- perfboard, from pcb/wiring.md ---
board = [70, 50];
hole_in = [1.98, 2.14];        // M2 hole centre from the board edges
pitch = 2.54;
col0_x = 5.79;                 // hole A1 from the left edge
row0_y = 3.41;                 // hole A1 from the top edge
pcb_t = 1.6;

// Centre of a perfboard hole, in case coordinates. col and row are zero-based (A = 0, row 1 = 0).
function hx(col) = -board.x / 2 + col0_x + pitch * col;
function hy(row) = board.y / 2 - row0_y - pitch * row;

// --- case ---
clear = 6;                     // free space around the board, holds the join posts
wall = 2.4;
stand_h = 3;                   // standoff height under the board
inner_h = 22;                  // free height above the floor: Nano on sockets (about 17), 470 uF capacitor (about 12)
split_h = 18;                  // height above the floor where the halves meet; above the wheels, so the lid has no overhangs when printed face down
post_d = 6;
join_pilot_d = 1.7;            // M2 self-tapping
join_clear_d = 2.4;
standoff_d = 5;

// --- parts above the board (estimates, see measurements.md) ---
socket_h = 8.5;                // Nano pin socket
nano_t = 1.6;

// --- Rhino details ---
wheel_d = 18;
wheel_t = 4;
wheel_z = 9;
ridge = [2, 0.8];              // armour groove on the lid: width, depth (grooves, not ridges: the lid prints face down)

inner = [board.x + 2 * clear, board.y + 2 * clear];
outer = [inner.x + 2 * wall, inner.y + 2 * wall];
total_h = wall + inner_h + wall;
board_top_z = wall + stand_h + pcb_t;

assert(split_h > wall + stand_h + pcb_t, "split must be above the board");
assert(split_h >= wheel_z + wheel_d / 2, "wheels must end below the split");
assert(inner_h > stand_h + pcb_t + socket_h + nano_t + 5, "not enough height for the Nano on its sockets");
assert(outer.x < 200 && outer.y + 2 * wheel_t < 200, "does not fit a library printer bed");
// Join posts must not touch the board's corners.
assert(norm([inner.x / 2 - post_d / 2 - board.x / 2, inner.y / 2 - post_d / 2 - board.y / 2]) > post_d / 2 + 1,
       "join post too close to the board corner");

// Posts sink 0.5 mm into the walls: a post that only touches a wall along a line makes a non-manifold mesh.
post_pos = [for (sx = [-1, 1], sy = [-1, 1]) [sx * (inner.x / 2 - post_d / 2 + 0.5), sy * (inner.y / 2 - post_d / 2 + 0.5)]];
standoff_pos = [for (sx = [-1, 1], sy = [-1, 1]) [sx * (board.x / 2 - hole_in.x), sy * (board.y / 2 - hole_in.y)]];

// Cutout through a wall: side = +1 or -1, centre along the wall, w = width, z0 = bottom, h = height.
// The y cuts also go through the wheels, so cables do not run into them.
module cut_x(side, y, w, z0, h)
  translate([side > 0 ? outer.x / 2 - wall - 0.5 : -outer.x / 2 - 1, y - w / 2, z0]) cube([wall + 1.5, w, h]);
module cut_y(side, x, w, z0, h)
  translate([x - w / 2, side > 0 ? outer.y / 2 - wall - 0.5 : -outer.y / 2 - wheel_t - 1, z0]) cube([w, wall + wheel_t + 1.5, h]);

module cutouts() {
  // Right wall: barrel jack (J3, rows 15 to 17) and the Nano's USB port (Nano rows 6 to 12, USB faces right).
  cut_x(1, hy(15), 14, board_top_z, 13);
  cut_x(1, hy(8), 13, board_top_z + socket_h - 2, 9);
  // Top wall: display cables J4, J5, J6 (centres at columns B..E, H..K, O..R).
  for (c = [2.5, 8.5, 15.5]) cut_y(1, hx(c), 14, board_top_z, 9);
  // Bottom wall: buttons J8 (B..E) and LED strip J7 (M..O).
  for (c = [2.5, 13]) cut_y(-1, hx(c), 14, board_top_z, 9);
}

// Octagonal wheel: flat at the bottom, no overhang steeper than 45 degrees.
module wheel() rotate([90, 0, 0]) rotate([0, 0, 22.5]) cylinder(d = wheel_d, h = wheel_t, center = true, $fn = 8);

module wheels() {
  for (sy = [-1, 1], x = [-outer.x / 2 + 14, 0, outer.x / 2 - 14])
    translate([x, sy * (outer.y / 2 + wheel_t / 2 - 0.5), wheel_z]) wheel();
}

module posts() { for (p = post_pos) translate([p.x, p.y, wall]) cylinder(d = post_d, h = inner_h, $fn = 32); }

module standoffs() {
  for (p = standoff_pos) translate([p.x, p.y, wall - 0.01]) difference() {
    cylinder(d = standoff_d, h = stand_h + 0.01, $fn = 32);
    translate([0, 0, 0.5]) cylinder(d = join_pilot_d, h = stand_h, $fn = 24);
  }
}

module shell() {
  difference() {
    union() {
      translate([-outer.x / 2, -outer.y / 2, 0]) cube([outer.x, outer.y, total_h]);
      wheels();
    }
    translate([-inner.x / 2, -inner.y / 2, wall]) cube([inner.x, inner.y, inner_h]);
    cutouts();
    // Front hood: small chamfer on the top-left edge.
    translate([-outer.x / 2, 0, total_h]) rotate([0, 45, 0]) cube([3, outer.y + 1, 3], center = true);
  }
}

module bottom_half() {
  difference() {
    union() {
      intersection() { union() { shell(); posts(); } translate([-100, -100, -1]) cube([200, 200, split_h + 1]); }
      standoffs();
    }
    for (p = post_pos) translate([p.x, p.y, split_h - 8]) cylinder(d = join_pilot_d, h = 9, $fn = 24);
  }
}

module top_half() {
  difference() {
    intersection() { union() { shell(); posts(); } translate([-100, -100, split_h]) cube([200, 200, total_h]); }
    // Armour grooves across the lid.
    for (x = [-15, 0, 15]) translate([x - ridge[0] / 2, -outer.y / 2 + 4, total_h - ridge[1]]) cube([ridge[0], outer.y - 8, ridge[1] + 1]);
    for (p = post_pos) translate([p.x, p.y, split_h - 1]) cylinder(d = join_clear_d, h = total_h, $fn = 24);
  }
}

if (part == "bottom") bottom_half();
else if (part == "top") translate([0, 0, total_h]) rotate([180, 0, 0]) top_half();   // lid face down on the bed for printing
else { bottom_half(); translate([0, 0, 0]) top_half(); }
