use bcsk::{BinaryCountSketch, TestItem};
use std::{env, collections::HashSet};

fn main() {

    let args: Vec<String> = env::args().collect();

    let base_lenth : u64 = args[1].parse().expect("Base lengh as u64");
    let level : u64 = args[2].parse().expect("Level as u64");
    let point : u64 = args[3].parse().expect("Point as u64");
    let common_num : u64 = args[4].parse().expect("Common items as u64");
    let uncommon_num : u64 = args[5].parse().expect("Non common items as u64");
    let samples_num : u64 = args[6].parse().expect("Stats samples as u64");
    let threshold : u64 = args[7].parse().expect("Min threshold as u64");
    

    let mut sketch1 = BinaryCountSketch::new(base_lenth, level, point);
    let mut sketch2 = BinaryCountSketch::new(base_lenth, level, point);

    // Add to filter
    let mut common = vec![];
    for _ in 0..common_num {
        let item: TestItem = TestItem::new();
        sketch1.toggle(&item);
        sketch2.toggle(&item);
        common.push(item);
    }

    let mut extra1 = vec![];
    for _ in 0..uncommon_num {
        let item: TestItem = TestItem::new();
        sketch1.toggle(&item);
        extra1.push(item);
        let item: TestItem = TestItem::new();
        sketch2.toggle(&item);
    }

    sketch2.diff_with(&sketch1).expect("No errors");
    let (fpos, fneg) = sketch2.estimate_stats(samples_num as usize, threshold as usize).expect("No errors");

    let mut candidates = vec![];
    candidates.append(&mut common.clone());
    candidates.append(&mut extra1.clone());

    let mut found = Vec::new();
    
    println!("{} bits {} bytes", sketch2.bits(), sketch2.bits() / 8);

    println!("Naive scheme: {} bytes", 8 * (uncommon_num + common_num) );
    println!("IBLT scheme: {} bytes", 4 * uncommon_num * 24 );

    println!("Estimate TP rate: {} / {}", samples_num as usize - fneg, samples_num);
    println!("Estimate FP rate:  {} / {}", fpos, samples_num);

    let mut tmp_threshold = point;

    loop {
        let mut not_found = Vec::new();
        for (score, item) in sketch2.decode(&candidates).into_iter().zip(&candidates) {
            if score >= tmp_threshold as usize {
                found.push(item.clone());
                sketch2.toggle(item);
             
            }
            else {
            not_found.push(item.clone());
            }
        }

        println!("Decoded {} Remaining {}", found.len(), not_found.len() );

        if not_found.len() == candidates.len() {
            if tmp_threshold > threshold {
                tmp_threshold -= 1;
            } else 
            {
                break
            }
        }

        
        candidates = not_found;
    }

    let extra_set : HashSet<_> = extra1.clone().into_iter().collect();

    println!("Found: {}", found.len());

    println!(
        "Common TP rate: {}",
        found.iter().filter(|item| extra_set.contains(item)).count() as f64
            / uncommon_num as f64
    );

    println!(
        "Common FP rate: {}",
        found.iter().filter(|item| !extra_set.contains(item)).count() as f64
            / common_num as f64
    );

}
