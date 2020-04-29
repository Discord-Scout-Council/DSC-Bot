/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

pub fn get_level_cost(level: u32) -> u64 {
    let cost = 10f64 * 1.3f64.powf(level as f64);

    cost.round() as u64
}

pub fn get_level_from_points(points: u64) -> u32 {
    let points = points as f64;
    let level = (points / 10f64).log(1.3f64).floor();

    level.round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_level_cost() {
        let actual_cost = 250u64;
        assert_eq!(actual_cost, super::get_level_cost(2));
    }
    #[test]
    fn level_from_points() {
        let actual_level = 2u32;
        assert_eq!(actual_level, super::get_level_from_points(251u64));
    }
}
