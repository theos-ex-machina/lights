# Notes on how lights work

## DMX Frame Structure
each square bracket is a byte
`[ BREAK ] [ MAB ] [ START CODE ] [ DATA SLOT 1 ] [ DATA SLOT 2] ... [ DATA SLOT 512 ]`
DMX is a master-slave relationship; nothing is recived, it's only sent from the controller

- BREAK is what says its the start of a new frame
- MAB is mark after break; signifies end of break
- START CODE is the first byte actually transmitted in the frame
  - 0x00 = lighting data
  - 0xcc = RDM data (remote device management)
  - 0x17 = text packets
  - everything else is random manufacturer shit
- DATA SLOTS 
  - each light has a start address, and the number of channels that it uses


## RDM Structure
RDM stands for remote device management

- Send and recive information about lights
  - ask and response structure

### Magic Numbers
- 0x00F0 → DMX_START_ADDRESS
  - Read this to know where the fixture is listening (e.g. 50).

- 0x00E0 → DMX_PERSONALITY
  - Fixtures can have multiple modes (e.g. 8-channel mode, 16-channel mode).
  - This tells you which mode it’s in.

- 0x00E1 → DMX_PERSONALITY_DESCRIPTION
  -Tells you what each mode means (e.g. "8ch: RGB + dimmer", "16ch: RGBW + strobe + macros").

- 0x00E2 → DMX_SLOT_DESCRIPTION
  - Gives per-channel details (slot 1 = Pan, slot 2 = Tilt, slot 3 = Dimmer, etc.).

0x00E3 → DMX_SLOT_DEFAULT
  - Default values for each channel.