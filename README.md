a minimal environment for prototyping KCCs with procedural and customizable 'level' elements.
# KCC-Prototype
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

