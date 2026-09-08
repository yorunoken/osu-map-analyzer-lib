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

