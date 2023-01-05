#![feature(test)]

use std::fmt;
use std::error::Error;

extern crate test;

pub trait Item {
    fn get_code(&self, i: u64) -> usize;
}

#[derive(Debug)]
pub struct BinaryCountSketchError { details: String }

impl BinaryCountSketchError {
    pub fn new(details:&str) -> Self {
        BinaryCountSketchError { details: details.to_string() }
    }
}

impl fmt::Display for BinaryCountSketchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sketch Error: {}", self.details)
    }
}

impl Error for BinaryCountSketchError {}

pub struct BinaryCountSketch {
    base_length: u64,
    level: u64,
    points: u64,
    words: Vec<u64>,
}

impl BinaryCountSketch {
    pub fn new(base_length: u64, level: u64, points: u64) -> Self {
        BinaryCountSketch {
            base_length,
            level,
            points,
            words: vec![0; (base_length << level) as usize],
        }
    }

    pub fn bits(&self) -> usize {
        self.words.len() * 64
    }

    pub fn level_down(&self, new_level: u64) -> Result<Self,BinaryCountSketchError> {
        if !(new_level < self.level) { return Err(BinaryCountSketchError::new("Incorrect level")); }

        let mut new_words = vec![0; (self.base_length << new_level) as usize];
        let l = new_words.len();

        for (i, val) in self.words.iter().enumerate() {
            new_words[i % l] ^= *val;
        }

        Ok(BinaryCountSketch {
            base_length: self.base_length,
            level: new_level,
            points: self.points,
            words: new_words,
        })
    }

    pub fn diff_with(&mut self, other: &Self) -> Result<(),BinaryCountSketchError> {
        if !(self.base_length == other.base_length) { return Err(BinaryCountSketchError::new("Incorrect base length")); }
        if !(self.level == other.level) { return Err(BinaryCountSketchError::new("Incorrect level")); }
        if !(self.points == other.points) { return Err(BinaryCountSketchError::new("Incorrect points")); }
        if !(self.words.len() == other.words.len()) { return Err(BinaryCountSketchError::new("Incorrect words length")); }

        for (i, val) in other.words.iter().enumerate() {
            self.words[i] ^= *val;
        }

        Ok(())
    }

    pub fn toggle<V: Item>(&mut self, v: &V) {
        let l = self.words.len() * 64;
        for i in 0..self.points {
            let b = v.get_code(i) % l;
            self.words[b / 64] ^= 1 << (b % 64);
        }
    }

    pub fn check<V: Item>(&self, v: &V) -> usize {
        let l = self.words.len();

        (0..self.points)
            .into_iter()
            .map(|i| {
                let b = v.get_code(i) % (l * 64);
                if self.words[b / 64] & (1 << (b % 64)) != 0 {
                    1usize
                } else {
                    0usize
                }
            })
            .sum()
    }

    pub fn decode<V: Item>(&self, items: &[V]) -> Vec<usize> {
        items.iter().map(|item| self.check(item)).collect()
    }

    pub fn estimate_stats(&self, samples: usize, threshold: usize) -> Result<(usize, usize), BinaryCountSketchError> {
        if !(threshold <= self.points as usize) { return Err(BinaryCountSketchError::new("Incorrect threshold")); }

        struct Rand;
        impl Item for Rand {
            fn get_code(&self, _i: u64) -> usize {
                rand::random::<usize>()
            }
        }
        let r = Rand;

        let mut false_pos = 0;
        let mut false_neg = 0;
        for _ in 0..samples {
            let t = self.check(&r);
            if t >= threshold {
                false_pos += 1;
            }
            if (self.points as usize) - t < threshold {
                false_neg += 1;
            }
        }

        Ok((false_pos, false_neg))
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TestItem {
    points: Vec<usize>,
}

impl TestItem {
    pub fn new() -> Self {
        TestItem {
            points: vec![
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
                rand::random::<usize>(),
            ],
        }
    }
}

impl Item for TestItem {
    fn get_code(&self, i: u64) -> usize {
        self.points[i as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_basics() {
        let item: TestItem = TestItem::new();
        let mut sketch = BinaryCountSketch::new(10, 6, 3);

        // Check empty filter
        assert_eq!(sketch.check(&item), 0);

        // Add to filter
        sketch.toggle(&item);
        assert_eq!(sketch.check(&item), 3);

        // Remove from filter
        sketch.toggle(&item);
        assert_eq!(sketch.check(&item), 0);
    }

    #[test]
    fn test_decode() {
        let item: TestItem = TestItem::new();
        let mut sketch = BinaryCountSketch::new(10, 6, 3);

        // Check empty filter
        assert_eq!(sketch.decode(&[item.clone()]), vec![0]);

        // Add to filter
        sketch.toggle(&item);
        assert_eq!(sketch.decode(&[item.clone()]), vec![3]);

        // Remove from filter
        sketch.toggle(&item);
        assert_eq!(sketch.decode(&[item.clone()]), vec![0]);
    }

    #[test]
    fn test_stats() {
        let item: TestItem = TestItem::new();
        let mut sketch = BinaryCountSketch::new(10, 6, 3);

        // Add to filter
        sketch.toggle(&item);
        assert_eq!(sketch.decode(&[item.clone()]), vec![3]);

        let (fpos, fneg) = sketch.estimate_stats(100, 2).expect("No errors");
        assert!(fpos < 5);
        assert!(fneg < 5)
    }

    #[test]
    fn test_diff() {
        let item: TestItem = TestItem::new();
        let item2: TestItem = TestItem::new();
        let item3: TestItem = TestItem::new();
        let mut sketch1 = BinaryCountSketch::new(10, 6, 3);
        let mut sketch2 = BinaryCountSketch::new(10, 6, 3);

        // Add to filter
        sketch1.toggle(&item);
        sketch1.toggle(&item2);
        sketch2.toggle(&item);
        sketch2.toggle(&item3);
        assert_eq!(sketch1.decode(&[item.clone()]), vec![3]);

        sketch1.diff_with(&sketch2).expect("No errors");
        assert_eq!(sketch1.decode(&[item.clone()]), vec![0]);
        assert_eq!(sketch1.decode(&[item2.clone()]), vec![3]);
        assert_eq!(sketch1.decode(&[item3.clone()]), vec![3]);
    }

    #[test]
    fn test_stats_bad() {
        let mut sketch = BinaryCountSketch::new(1, 0, 3);

        // Add to filter
        for _ in 0..162 {
            let item: TestItem = TestItem::new();
            println!("{:?}", item.points);
            sketch.toggle(&item);
        }

        assert!(sketch.words.len() == 1);
        assert!(sketch.words[0] != 0);

        let (fpos, fneg) = sketch.estimate_stats(100, 2).expect("No errors");
        assert!(fpos > 10);
        assert!(fneg > 10)
    }

    #[test]
    fn test_diff_decode() {
        let mut sketch1 = BinaryCountSketch::new(100, 2, 5);
        let mut sketch2 = BinaryCountSketch::new(100, 2, 5);

        // Add to filter
        let mut common = vec![];
        for _ in 0..16200 {
            let item: TestItem = TestItem::new();
            sketch1.toggle(&item);
            sketch2.toggle(&item);
            common.push(item);
        }

        let mut extra1 = vec![];
        for _ in 0..162 {
            let item: TestItem = TestItem::new();
            sketch1.toggle(&item);
            extra1.push(item);
            let item: TestItem = TestItem::new();
            sketch2.toggle(&item);
        }

        sketch2.diff_with(&sketch1).expect("No errors");
        let (fpos, fneg) = sketch2.estimate_stats(100, 4).expect("no errors");

        println!("{} bits {} bytes", sketch2.bits(), sketch2.bits() / 8);
        println!("{} {}", fpos, fneg);
        println!(
            "{:?}",
            sketch2
                .decode(&extra1)
                .into_iter()
                .filter(|x| *x >= 4)
                .count() as f64
                / 162.0
        );
        println!(
            "{:?}",
            sketch2
                .decode(&common)
                .into_iter()
                .filter(|x| *x >= 4)
                .count() as f64
                / 16200.0
        );

        assert!(fpos < 10);
        assert!(fneg < 10);
    }

    #[bench]
    fn bench_toggle(b: &mut Bencher) {
        let item = TestItem::new();
        let mut sketch1 = BinaryCountSketch::new(100, 2, 5);

        b.iter(|| {
            let _n = test::black_box(1000);
            sketch1.toggle(&item);
        });
    }

    #[bench]
    fn bench_check(b: &mut Bencher) {
        let item = TestItem::new();
        let mut sketch1 = BinaryCountSketch::new(100, 2, 5);
        sketch1.toggle(&item);

        b.iter(|| {
            let _n = test::black_box(1000);
            sketch1.check(&item);
        });
    }

    #[bench]
    fn bench_decode(b: &mut Bencher) {
        let items: Vec<_> = (1..1000).into_iter().map(|_| TestItem::new()).collect();
        let mut sketch1 = BinaryCountSketch::new(100, 2, 5);

        for item in items.clone() {
            sketch1.toggle(&item);
        }

        b.iter(|| {
            let _n = test::black_box(1000);
            sketch1.decode(&items);
        });
    }
}
