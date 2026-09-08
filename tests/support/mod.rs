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
    format!("{x},{y},{time},2,0,L|{end_x}:{end_y},1,100")
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
