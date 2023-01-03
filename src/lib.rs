pub trait Item {
    fn get_code(&self, i: u64) -> usize;
}

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

    pub fn level_down(&self, new_level: u64) -> Self {
        assert!(new_level < self.level);

        let mut new_words = vec![0; (self.base_length << new_level) as usize];
        let l = new_words.len();

        for (i, val) in self.words.iter().enumerate() {
            new_words[i % l] ^= *val;
        }

        BinaryCountSketch {
            base_length: self.base_length,
            level: new_level,
            points: self.points,
            words: new_words,
        }
    }

    pub fn diff_with(&mut self, other: &Self) {
        assert!(self.base_length == other.base_length);
        assert!(self.level == other.level);
        assert!(self.points == other.points);
        assert!(self.words.len() == other.words.len());

        for (i, val) in other.words.iter().enumerate() {
            self.words[i] ^= *val;
        }
    }

    pub fn toggle<V: Item>(&mut self, v: &V) {
        let l = self.words.len();
        for i in 0..self.points {
            let b = v.get_code(i) % l * 64;
            self.words[b / 64] ^= 1 << (b % 64);
        }
    }

    pub fn check<V: Item>(&self, v: &V) -> usize {
        let mut count = 0;
        let l = self.words.len();
        for i in 0..self.points {
            let b = v.get_code(i) % l * 64;
            if self.words[b / 64] & (1 << (b % 64)) != 0 {
                count += 1;
            }
        }
        count
    }

    pub fn decode<V: Item>(&self, items: &[V]) -> Vec<usize> {
        items.iter().map(|item| self.check(item)).collect()
    }

    pub fn estimate_stats(&self, samples: usize, threshold: usize) -> (usize, usize) {
        assert!(threshold <= self.points as usize);

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
            if t >= (self.points as usize) - threshold {
                false_neg += 1;
            }
        }

        println!("{} {}", false_pos, false_neg);
        (false_pos, false_neg)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestItem {
        points: Vec<usize>,
    }

    impl TestItem {
        fn new() -> Self {
            TestItem {
                points: vec![
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

        let (fpos, fneg) = sketch.estimate_stats(100, 2);
        assert!(fpos < 5);
        assert!(fneg < 5)
    }
}
