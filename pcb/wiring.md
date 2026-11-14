# Biggus Darkus: perfboard build guide

Generated from `pcb/biggus-darkus-pcb.kicad_pcb`. The KiCad PCB is a **plan for a perfboard**: every track is a wire that you solder by hand. The DRC of the PCB shows 0 shorts, 0 clearance errors and 0 unconnected pads.

## 1. The board

| Item | Value |
|---|---|
| Type | Perfboard with a copper pad around every hole |
| Size | 70 x 50 mm |
| Holes | 24 columns (**A** to **X**) x 18 rows (**1** to **18**) |
| Hole pitch | **2.54 mm** (0.1 inch), in both directions |
| Hole A1 (first) | 5.79 mm from the left edge, 3.41 mm from the top edge |
| Hole X18 (last) | 5.79 mm from the right edge, 3.41 mm from the bottom edge |
| Border (hole centre to edge) | 5.79 mm left and right, 3.41 mm top and bottom |
| Hole centre position | x = 5.79 + 2.54 x c, y = 3.41 + 2.54 x r, from the top left corner, with c = 0 for column A and r = 0 for row 1 |
| Hole address | letter = column, number = row: **B6** is the second column, sixth row |
| Mounting holes (M2, 2.2 mm) | four corners, 1.98 mm from the left/right edge and 2.14 mm from the top/bottom edge, half a pitch outside the hole grid |

The board has 18 rows (confirmed), so the borders above are exact.

## 2. How the wires are made

- **Back side: bare wire, no insulation.** It lies flat on the back along a row or column and is soldered directly to the pads of the holes it touches. The pins of the parts are soldered in the same holes. Every hole on the wire belongs to that net, so use none of them for another signal.
- **Front side: insulated wire, used as jumper connectors.** It lies on the component side and passes through the holes marked *(via)*, where it is soldered to the pad on the back. Being insulated, it can cross rails and other wires.
- **(via)** = a hole where a front wire goes through the board and is soldered on the back to a bare wire or pad.
- **[J4.1]** after a hole = the pin of that part in the hole.

## 3. Parts and their holes

| Part | Value | Footprint | Pin: hole (net) |
|---|---|---|---|
| C1 | 470 µF | `CP_Radial_D6.3mm_P2.50mm` | 1: R15 (+5V); 2: R16 (GND) |
| D1 | LED | `LED_D5.0mm` | 1: X11 (GND); 2: W11 (Net-(D1-A)) |
| J3 | Barrel_Jack_Switch | `BarrelJack_Horizontal` | 1: V15 (+5V); 2: V17 (GND); 3: X16 (*D3) |
| J4 | TM1637_A | `JST_XH_S4B-XH-A_1x04_P2.50mm_Horizontal` | 1: E2 (D7); 2: D2 (D8); 3: C2 (+5V); 4: B2 (GND) |
| J5 | TM1637_B | `JST_XH_S4B-XH-A_1x04_P2.50mm_Horizontal` | 1: K2 (*D5); 2: J2 (*D6); 3: I2 (+5V); 4: H2 (GND) |
| J6 | LCD1602_I2C | `JST_XH_S4B-XH-A_1x04_P2.50mm_Horizontal` | 1: R2 (A5/SCL); 2: Q2 (A4/SDA); 3: P2 (+5V); 4: O2 (GND) |
| J7 | WS2812B_STRIP | `JST_XH_S3B-XH-A_1x03_P2.50mm_Horizontal` | 1: M17 (+5V); 2: N17 (Net-(J7-Pin_2)); 3: O17 (GND) |
| J8 | BUTTONS | `JST_XH_S4B-XH-A_1x04_P2.50mm_Horizontal` | 1: B17 (A0); 2: C17 (A1); 3: D17 (A2); 4: E17 (GND) |
| R1 | 330 | `R_Axial_DIN0207_L6.3mm_D2.5mm_P10.16mm_Horizontal` | 1: M14 (Net-(J7-Pin_2)); 2: I14 (A3) |
| R2 | 1 kΩ | `R_Axial_DIN0207_L6.3mm_D2.5mm_P10.16mm_Horizontal` | 1: W5 (+5V); 2: W9 (Net-(D1-A)) |
| J1 | Nano_Analog | `PinSocket_1x15_P2.54mm_Vertical` | row 12, B12 to P12, see the Nano table |
| J2 | Nano_Digital | `PinSocket_1x15_P2.54mm_Vertical` | row 6, B6 to P6, see the Nano table |

The JST XH connectors have a 2.5 mm pitch, so their last pin is up to 0.12 mm off the hole centre. The barrel jack J3 does not fit the grid (up to 0.9 mm off): check its real pin spacing before you solder. C1: the long leg is plus (pin 1).

### Nano pins

| Pin | J2 hole | J2 signal | J1 hole | J1 signal |
|---|---|---|---|---|
| 1 | B6 | not wired | B12 | not wired |
| 2 | C6 | not wired | C12 | not wired |
| 3 | D6 | not wired | D12 | not wired |
| 4 | E6 | not wired | E12 | A0 |
| 5 | F6 | D8 | F12 | A1 |
| 6 | G6 | D7 | G12 | A2 |
| 7 | H6 | *D6 | H12 | A3 |
| 8 | I6 | *D5 | I12 | A4/SDA |
| 9 | J6 | not wired | J12 | A5/SCL |
| 10 | K6 | *D3 | K12 | not wired |
| 11 | L6 | not wired | L12 | not wired |
| 12 | M6 | GND | M12 | +5V |
| 13 | N6 | not wired | N12 | not wired |
| 14 | O6 | not wired | O12 | not wired |
| 15 | P6 | not wired | P12 | not wired |

## 4. Signal wires, from pin to pin

Each signal is a chain of back wires (bare) and front wires (insulated). Read each row from left to right.

| Signal | Route | Length (mm) |
|---|---|---|
| *D3 | **back**: K6 [J2.10] > K7 > Q7 (via) ; **front**: Q7 (via) > T7 (via) ; **back**: T7 (via) > T13 > U13 (via) ; **front**: U13 (via) > W13 (via) ; **back**: W13 (via) > X13 ; **back**: X13 > X16 [J3.3] | 59.0 |
| *D5 | **back**: I6 [J2.8] > I5 (via) ; **front**: I5 (via) > K5 > K3 (via) ; **back**: K3 (via) > K2 [J5.1] | 15.2 |
| *D6 | **back**: H6 [J2.7] > H5 (via) ; **front**: H5 (via) > H4 > J4 > J3 (via) ; **back**: J3 (via) > J2 [J5.2] | 15.2 |
| A0 | **back**: E12 [J1.4] > E14 > B14 > B17 [J8.1] | 20.3 |
| A1 | **back**: C17 [J8.2] > C16 ; **back**: C16 > C15 > F15 > F12 [J1.5] | 20.3 |
| A2 | **back**: D17 [J8.3] > D16 ; **back**: D16 > G16 > G12 [J1.6] | 20.2 |
| A3 | **back**: I14 [R1.2] > H14 > H12 [J1.7] | 7.6 |
| A4/SDA | **back**: I12 [J1.8] > I10 > R10 > R5 (via) ; **front**: R5 (via) > R3 (via) ; **back**: R3 (via) > Q3 ; **back**: Q3 > Q2 [J6.2] | 50.8 |
| A5/SCL | **back**: R2 [J6.1] > S2 > S3 (via) ; **front**: S3 (via) > S5 (via) ; **back**: S5 (via) > S11 > J11 > J12 [J1.9] | 50.8 |
| D7 | **back**: E2 [J4.1] > E3 (via) ; **front**: E3 (via) > E4 > G4 > G5 (via) ; **back**: G5 (via) > G6 [J2.6] | 15.2 |
| D8 | **back**: D2 [J4.2] > D3 (via) ; **front**: D3 (via) > D5 > F5 (via) ; **back**: F5 (via) > F6 [J2.5] | 15.2 |
| Net-(D1-A) | **back**: W9 [R2.2] > W11 [D1.2] | 5.1 |
| Net-(J7-Pin_2) | **back**: M14 [R1.1] > N14 (via) ; **front**: N14 (via) > N16 (via) ; **back**: N16 (via) > N17 [J7.2] | 10.2 |

## 5. Power rails and their jumpers

Rails are back wires (bare). Every hole they touch is part of the net. Thick wire is needed where the strip current flows (jack to J7).

### GND

| Side | Route | Length (mm) |
|---|---|---|
| back, bare | B1 > A1 > A18 > O18 | 81.3 |
| back, bare | B2 [J4.4] > B1 | 2.4 |
| back, bare | E18 > E17 [J8.4] | 2.4 |
| back, bare | H1 > B1 | 15.2 |
| back, bare | H1 > M1 | 12.7 |
| back, bare | H1 > H2 [J5.4] | 2.4 |
| back, bare | M1 > M3 (via) | 5.1 |
| back, bare | M6 [J2.12] > M5 (via) | 2.5 |
| back, bare | O1 > N1 > M1 | 5.1 |
| back, bare | O1 > X1 > X11 [D1.1] | 48.3 |
| back, bare | O18 > R18 | 7.6 |
| back, bare | O18 > O17 [J7.3] | 2.5 |
| back, bare | O2 [J6.4] > O1 | 2.4 |
| back, bare | R18 > R16 [C1.2] | 5.1 |
| back, bare | V17 [J3.2] > U18 > R18 | 10.8 |
| front, insulated | M3 (via) > M5 (via) | 5.1 |

### +5V

| Side | Route | Length (mm) |
|---|---|---|
| back, bare | C4 > C2 [J4.3] | 5.0 |
| back, bare | I2 [J5.3] > I4 | 5.0 |
| back, bare | I4 > C4 | 15.2 |
| back, bare | M15 > P15 | 7.5 |
| back, bare | M15 > M17 [J7.1] | 5.0 |
| back, bare | P15 > P13 > M13 > M12 [J1.12] | 15.2 |
| back, bare | P4 > V4 > V15 [J3.1] > R15 [C1.1] > P15 | 58.4 |
| back, bare | P4 > P2 [J6.3] | 5.0 |
| back, bare | P4 > I4 | 17.8 |
| back, bare | V5 > W5 [R2.1] | 2.5 |

## 6. Wire needed (without slack)

- Back side, bare wire: about **579 mm** (rails: thicker wire for the strip path).
- Front side, insulated wire: about **74 mm** in total.
- Add a few millimetres of slack to each piece.

## 7. Build order

1. Mark the column letters (A to X) and row numbers (1 to 18) on the board edge. Count holes from A1, not from the edge.
2. Solder the Nano sockets (J2 on row 6 and J1 on row 12, columns B to P). Solder one pin per socket first, check both sit flat, then solder the rest. Plug the Nano in while you do this, so the rows stay 6 holes apart. Take the Nano out afterwards.
3. Solder the low parts: R1, R2, D1.
4. Solder the JST connectors (J4 to J8). Tack one pin, check the connector is flat and straight, then solder the rest. Cables pull on them, so use enough solder.
5. Solder C1 and the barrel jack J3 last. They are tall and make the board rock when flipped. Check the real pin spacing of the jack first, and use plenty of solder, because it takes the most force.
6. Back side, bare wire: the GND rails first (start with the long row 18 wire and the column A wire), then the row 1 wire and the short stubs.
7. Back side: the +5V rails (thick wire from the jack V15 to the strip connector M17). Then **stop and check there is no short between +5V and GND**.
8. Back side: the short stubs of each signal, one signal at a time. Check each wire touches only the holes of its own net.
9. Front side, insulated jumpers: last, because they cross other wires. One signal at a time, check each with a multimeter.
10. Before you plug in the Nano: check continuity of every net, and that the unused Nano pins (rows 6 and 12) have no solder bridges.
11. Power from the jack alone, check 5 V and the power LED, then plug in the Nano and test each module.

## 8. Checks before the first power-up

- The jack must be a **regulated 5 V** supply. A higher voltage destroys the modules.
- Check the pin order of each JST cable against the connector table (GND is last on the 4-pin connectors, and the third on the strip connector).
- With no plug inserted, find out which contact the jack's switch (pin 3) touches. The D3 plug detection needs the sleeve (GND).
- The Nano's two RESET pins and its other GND pins are joined inside the module. Only J2 pin 12 (GND) and J1 pin 12 (5V) are wired, and the rest are marked unused.
