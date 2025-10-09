# LIGHTS

Open source dmx lightboard software written in rust.

This started as me just reverse-engineering dmx and turned into a shitty lightboard software.

## Warning
not tested at all yet, because I wrote this while away from my auditorium

## To-Do
ordered loosely in order of priority

- [x] CLI
  - very basic cli done with threading
- [ ] Save & load patch to file
  - [ ] Should be EOS family compatible 
- [ ] Fades and blackouts
- [ ] select multiple lights at once
- [ ] Park Lights
  - [ ] for moving heads and scrollers
- [ ] Cue system
  - scene struct that stores DMX buffer
- [ ] GUI 
- [ ] Effects
- [ ] RDM? 
  - [ ] most of my lights don't support this so priority is very low