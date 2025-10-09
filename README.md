# LIGHTS

Open source DMX lightboard software written in Rust.

This started as me just reverse-engineering DMX and turned into a shitty lightboard software.

## Usage

```bash
cargo run
```

Basic CLI commands:
- `c 1 @ 255` - set channel 1 to full intensity  
- `c 5 rgb 255 0 0` - set channel 5 to red
- `a 10 @ 128` - set DMX address 10 directly
- `blackout` - turn off all lights

## Warning

Not tested at all yet, because I wrote this while away from my auditorium. I'll remove this section when I do test it.

## To-Do

Ordered loosely in order of priority:

- [x] **CLI** - very basic CLI done with threading
- [ ] **Save & load patch to file** - should be EOS family compatible 
- [ ] **Fades and blackouts**
- [ ] **Select multiple lights at once**
- [ ] **Park lights** - for moving heads and scrollers
- [ ] **Cue system** - scene struct that stores DMX buffer
- [ ] **GUI** 
- [ ] **Effects**
- [ ] **RDM?** - most of my lights don't support this so priority is very low