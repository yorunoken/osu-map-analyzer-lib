#![allow(dead_code)]

use osu_map_analyzer::rosu_map::Beatmap;

pub fn beatmap(timing_points: &[&str], hit_objects: &[String]) -> Beatmap {
    let mut source = String::from(
        "osu file format v14\n\n\
         [General]\n\
         Mode:0\n\n\
         [Difficulty]\n\
         CircleSize:4\n\
         SliderMultiplier:1.4\n\
         SliderTickRate:1\n\n\
         [TimingPoints]\n",
    );

    for point in timing_points {
        source.push_str(point);
        source.push('\n');
    }

    source.push_str("\n[HitObjects]\n");

    for object in hit_objects {
        source.push_str(object);
        source.push('\n');
    }

    Beatmap::from_bytes(source.as_bytes()).expect("test beatmap must parse")
}

pub fn circle(x: i32, y: i32, time: i32) -> String {
    format!("{x},{y},{time},1,0,0:0:0:0:")
}

pub fn slider(x: i32, y: i32, time: i32, end_x: i32, end_y: i32) -> String {
    slider_with_spans(x, y, time, end_x, end_y, 1, 100)
}

#[allow(clippy::too_many_arguments)]
pub fn slider_with_spans(
    x: i32,
    y: i32,
    time: i32,
    end_x: i32,
    end_y: i32,
    spans: usize,
    length: usize,
) -> String {
    format!("{x},{y},{time},2,0,L|{end_x}:{end_y},{spans},{length}")
}

pub fn circle_run(count: usize, interval_ms: f64, spacing: f64) -> Beatmap {
    let objects = (0..count)
        .map(|index| {
            let x = if index % 2 == 0 { 128.0 } else { 128.0 + spacing };
            circle(x.round() as i32, 192, (index as f64 * interval_ms).round() as i32)
        })
        .collect::<Vec<_>>();

    beatmap(&["0,500,4,2,1,50,1,0"], &objects)
}

pub fn separated_circle_runs(lengths: &[usize], interval_ms: f64, gap_ms: f64) -> Beatmap {
    let mut objects = Vec::new();
    let mut time: f64 = 0.0;

    for (run_index, &length) in lengths.iter().enumerate() {
        for note_index in 0..length {
            let x = if note_index % 2 == 0 { 128 } else { 168 };
            objects.push(circle(x, 192, time.round() as i32));
            time += interval_ms;
        }

        if run_index + 1 < lengths.len() {
            time += gap_ms;
        }
    }

    beatmap(&["0,500,4,2,1,50,1,0"], &objects)
}

pub fn jump_pair_with_cs(circle_size: f32, spacing: f64) -> Beatmap {
    let mut map = circle_run(2, 250.0, spacing);
    map.circle_size = circle_size;
    map
}

pub fn slider_map() -> Beatmap {
    beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[
            slider_with_spans(64, 192, 0, 164, 192, 1, 100),
            slider_with_spans(256, 192, 500, 356, 192, 2, 100),
            slider_with_spans(64, 192, 1_000, 164, 192, 2, 100),
        ],
    )
}

pub fn technical_map() -> Beatmap {
    let times = [0, 250, 375, 625, 750, 1_125, 1_250, 1_500, 1_625];
    let positions = [
        (64, 64),
        (256, 64),
        (256, 256),
        (448, 256),
        (448, 64),
        (256, 64),
        (256, 256),
        (64, 256),
        (64, 64),
    ];
    let objects = times
        .into_iter()
        .zip(positions)
        .enumerate()
        .map(|(index, (time, (x, y)))| {
            if index == 3 || index == 6 {
                slider_with_spans(x, y, time, x + 80, y, 2, 80)
            } else {
                circle(x, y, time)
            }
        })
        .collect::<Vec<_>>();

    beatmap(
        &[
            "0,500,4,2,1,50,1,0",
            "500,-50,4,2,1,50,0,0",
            "1000,-100,4,2,1,50,0,0",
        ],
        &objects,
    )
}

pub fn mixed_map() -> Beatmap {
    let mut objects = (0..8)
        .map(|index| circle(128 + (index % 2) * 40, 192, index * 125))
        .collect::<Vec<_>>();
    objects.extend((0..6).map(|index| {
        circle(
            64 + (index % 2) * 220,
            192,
            2_000 + index * 250,
        )
    }));
    objects.push(slider_with_spans(64, 192, 4_000, 264, 192, 2, 200));

    beatmap(&["0,500,4,2,1,50,1,0"], &objects)
}
