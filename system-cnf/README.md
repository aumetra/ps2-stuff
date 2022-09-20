# system-cnf

Library for encoding and decoding PS2 `SYSTEM.CNF` files.

## Example

```rust
use system_cnf::{SystemCnf, VideoMode};

let my_system_cnf = SystemCnf {
    elf_path: "cdrom:\\main.elf".into(),
    version: "1.0".into(),
    video_mode: VideoMode::Pal,
    hdd_unit_power: None,
};
println!("{my_system_cnf}"); // Prints the serialised version of the struct
```
