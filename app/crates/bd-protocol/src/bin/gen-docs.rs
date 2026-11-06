//! Writes `docs/protocol.md` from the spec.

use std::{fs, io};

fn main() -> io::Result<()> {
    fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/protocol.md"),
        bd_protocol::render_markdown(),
    )
}
