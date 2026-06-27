# Manual labeling guide

Manual labels are multi-label: one map can test several skills. Assign 1–5 labels per map, and prefer fewer confident labels over many vague ones. Label the skill the map actually tests; do not infer labels from star rating alone.

`deterministic_tags` are analyzer output and must remain separate from `manual_labels`. Edit only `manual_labels` in generated label templates. ML training should use `manual_labels`, not `deterministic_tags`, unless you intentionally want to test a deterministic-tag baseline.

## Core skill labels

- `Aim`: Raw aim requirement, usually jumps, spacing, cursor movement, or aim strain.
- `AimControl`: Awkward angles, flow or direction changes, spacing control, or precision movement rather than only large spacing.
- `Stream`: Meaningful continuous stream patterns, usually longer than bursts.
- `Burst`: Many short fast groups without necessarily containing long streams.
- `Speed`: High tapping-speed requirement in bursts, streams, or other fast patterns.
- `Stamina`: Sustained speed or stream pressure over longer sections.
- `FingerControl`: Awkward tapping, interval changes, doubles, triples, odd rhythms, or control-heavy tapping.
- `RhythmComplex`: Varied, irregular, or difficult-to-internalize rhythm.
- `Reading`: Density, overlap, unusual spacing or rhythm comprehension, or other visual parsing requirements.
- `Tech`: Technical mapping combining sliders, awkward rhythm, aim control, reading, or unusual patterns.
- `Slider`: Sliders are a meaningful part of the difficulty rather than filler.
- `Precision`: Small CS, tight spacing, high-OD precision, or accuracy/click-precision requirements.

## Map role and modifier labels

- `Farm`: Generally optimized for score or pp relative to difficulty.
- `AimFarm`: Specifically farm-oriented around raw or jump aim. Use this instead of both `Aim` and `Farm` when the map is specifically aim-farm.
- `StreamFarm`: Farm-oriented around streams, stamina, or speed consistency.
- `Consistency`: Rewards stable execution more than isolated spikes.
- `Awkward`: Patterns feel uncomfortable, unintuitive, or anti-flow.
- `Comfortable`: Patterns feel natural, clean, or easy to execute relative to difficulty.
- `AimSlop`: Very straightforward big/raw jump aim with simple rhythm and little aim-control or technical complexity.

## Dataset workflow

1. Export machine-generated features:

   ```bash
   cargo run -- export-features maps/ --out features.jsonl
   ```

2. Export the manual label template:

   ```bash
   cargo run -- export-label-template maps/ --out labels.jsonl
   ```

3. Edit only `labels.jsonl` and fill each `manual_labels` array.

4. Merge features and labels:

   ```bash
   cargo run -- merge-dataset --features features.jsonl --labels labels.jsonl --out training_dataset.jsonl
   ```

5. Use `training_dataset.jsonl` for ML training.

`features.jsonl` is machine-generated and should not be hand-edited. `labels.jsonl` is the small file humans edit. `training_dataset.jsonl` is generated from those two inputs and can be regenerated whenever features or labels change.

The label template starts with an empty `manual_labels` array. Add only names from the schema above, preserving the JSON array format.

## Mods and playback rate

`.osu` files are nomod source files. Each exported row describes a played version through `mods` and its effective `rate`. DT and NC default to `1.5`, HT defaults to `0.75`, and an explicit `--rate` overrides the implied rate while retaining the selected mods in metadata.

Export separate nomod and DT datasets when both versions need labels:

```bash
cargo run -- export-features maps/ --out features_nm.jsonl
cargo run -- export-label-template maps/ --out labels_nm.jsonl

cargo run -- export-features maps/ --mods DT --out features_dt.jsonl
cargo run -- export-label-template maps/ --mods DT --out labels_dt.jsonl

cargo run -- merge-dataset --features features_dt.jsonl --labels labels_dt.jsonl --out training_dt.jsonl
```

DT, NC, HT, and custom rates change rhythm gaps, BPM, NPS, density, speed, stamina, stream BPM, slider duration, and effective map length. Rate does not change object positions, jump distances, angles, circle size, or slider path shape.

Both `star_rating_nomod` and `star_rating_adjusted` are stored when calculation succeeds. A failed calculation is represented by `null`; the numeric feature vector exposes `star_rating_available` and uses `0.0` only as a documented fallback.

Manual labels must describe the played version represented by that row. NM and DT rows for the same beatmap can require different labels and remain distinct during merge because identity includes beatmap, mods, and rate.
