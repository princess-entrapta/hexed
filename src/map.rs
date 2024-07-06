use std::collections::HashMap;

use crate::schemas::{Coords, TileType};

pub fn from_text(text_map: &str) -> HashMap<Coords, TileType> {
    let mut map = HashMap::new();
    let lines = text_map.split("\n");
    let mut y = 0;
    for line in lines {
        let mut x = 0;
        if y % 2 == 1 {
            x = 1
        }
        for char in line.chars() {
            match char {
                '.' => {
                    map.insert(Coords { x, y }, TileType::Floor);
                }
                'g' => {
                    map.insert(Coords { x, y }, TileType::TallGrass);
                }
                '=' => {
                    map.insert(Coords { x, y }, TileType::DeepWater);
                }
                '#' => {
                    map.insert(Coords { x, y }, TileType::Wall);
                }
                _ => {}
            }
            x += 2
        }
        y += 1;
    }
    map
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_text() {
        let hm = from_text("  ...\n ....\n .....\n..gg..\n..g#g..\n..gg..\n .....\n ....\n  ...");
        assert_eq!(hm.get(&Coords { y: 4, x: 6 }), Some(&TileType::Wall));
        assert_eq!(hm.len(), 43);
    }
}
