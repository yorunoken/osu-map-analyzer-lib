pub mod analyze;
mod utils;

pub use rosu_map;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::analyze;

    #[test]
    fn test_jump_map() {
        let path = Path::new("example-maps/jump-flowerdance.osu");
        let map = rosu_map::from_path::<rosu_map::Beatmap>(path).unwrap();

        println!("Testing map: {} - {}", map.title, map.artist);

        let mut stream_analyzer = analyze::Stream::new(map.clone());
        let stream_analasis = stream_analyzer.analyze();
        println!("stream: \n{:#?}", stream_analasis);

        let mut jump_analyzer = analyze::Jump::new(map);
        let jump_analasis = jump_analyzer.analyze();
        println!("jump: \n{:#?}", jump_analasis);
    }

    #[test]
    fn test_stream_map() {
        let path = Path::new("example-maps/stream-honesty.osu");
        let map = rosu_map::from_path::<rosu_map::Beatmap>(path).unwrap();

        println!("Testing map: {} - {}", map.title, map.artist);

        let mut stream_analyzer = analyze::Stream::new(map.clone());
        let stream_analasis = stream_analyzer.analyze();
        println!("stream: \n{:#?}", stream_analasis);

        let mut jump_analyzer = analyze::Jump::new(map);
        let jump_analasis = jump_analyzer.analyze();
        println!("jump: \n{:#?}", jump_analasis);
    }
}
