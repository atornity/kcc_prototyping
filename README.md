a minimal environment for prototyping KCCs with procedural and customizable 'level' elements.
# KCC-Prototype
### Input Bindings

#### General (DefaultContext)
- **Toggle Camera Mode**: `F1` or gamepad `D-Pad Up` (just pressed)
- **Move**: `WASD` or gamepad left stick
- **Look**: Mouse motion (scaled & inverted) or gamepad right stick (X inverted)
- **Jump**: `Space` or gamepad `East` (just pressed)
- **Capture Cursor**: Left mouse button
- **Release Cursor**: `Escape`

#### Fly Camera (FlyCameraContext)
- **Fly Up**: `Left Shift` or gamepad `East`
- **Fly Down**: `Left Ctrl` or gamepad `Left Thumb`

#### Orbit Camera (OrbitCameraContext)
- **Zoom**: Mouse wheel

### Notes
- The environment elements are procedural and defined via constants (with PARAMS) in the corresponding plugin files.
`Params` define ranges of values, for which all permutations are generated and spawned in the level.
`level/tracks/ramps.rs`:
```rs
const PARAMS: &[(&str, Param)] = &[
    (
        "length",
        Param::Float {
            start: 4.0,
            end: 8.0,
            step: 4.0,
        },
    ),
    (
        "angle",
        Param::Float {
            start: 5.0,
            end: 80.0,
            step: 15.0,
        },
    ), // Angle in degrees
];
```


https://github.com/user-attachments/assets/c7c6b18d-d8bb-4e1a-9605-4be2c293b31b

