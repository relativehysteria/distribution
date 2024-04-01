use std::collections::HashMap;
use std::thread;
use std::io::Write;
use std::fs::File;
use std::process::Command;
use core::arch::x86_64::_rdtsc;
use distribution::rng::Rng;


fn toss(rng: &mut Rng, n_tosses: usize) -> Vec<u128> {
    (0..n_tosses).map(|_| (rng.rand() % 10 + 1) as u128).collect()
}

fn create_histogram(values: Vec<u128>) -> Vec<(u128, usize)> {
    let mut histogram: HashMap<u128, usize> = HashMap::new();
    values.iter().for_each(|&v| { *histogram.entry(v).or_insert(0) += 1; });

    let mut histogram: Vec<(u128, usize)> = histogram.into_iter().collect();
    histogram.sort_unstable_by_key(|&(v, _)| core::cmp::Reverse(v));
    histogram
}

fn proof<F>(dataset_size: usize, n_tosses: usize, operation: F) -> String
where
    F: Fn(&[u128]) -> u128,
{
    let mut rng = Rng::new(unsafe { _rdtsc() });
    let filename_root = format!("{:x}-ds_{dataset_size:x}-nt_{n_tosses:x}",
                                rng.rand());

    let histogram = (0..dataset_size)
        .map(|_| operation(&toss(&mut rng, n_tosses as usize)))
        .collect::<Vec<u128>>();
    let mut file = File::create(&format!("{}.txt", filename_root)).unwrap();

    for (key, value) in create_histogram(histogram) {
        file.write(format!("{} {}\n", key, value).as_bytes()).unwrap();
    }

    println!("Data written to {filename_root}.txt!");
    filename_root
}

fn main() {
    let set = 1_000_000;

    let s  = thread::spawn(move || { proof(set, 500, |h| h.iter().sum()) });
    let p  = thread::spawn(move || { proof(set, 10,  |h| h.iter().product()) });
    let lp = thread::spawn(move || { proof(set, 10,  |h| h.iter().product::<u128>().ilog10() as u128) });

    let sum_filename     = s.join().unwrap();
    let product_filename = p.join().unwrap();
    let logprod_filename = lp.join().unwrap();

    let plot_gp_data = format!(r#"set terminal png size 3000,3000

set output 'sum_{s}.png'
plot '{s}.txt' using 1:2 smooth frequency with filledcurve notitle

set output 'product_{p}.png'
plot '{p}.txt' using 1:2 smooth frequency with filledcurve notitle"

set output 'log_{lp}.png'
plot '{lp}.txt' using 1:2 smooth frequency with filledcurve notitle"#,
        s = sum_filename,
        p = product_filename,
        lp = logprod_filename);

    let mut file = File::create("plot.gp").unwrap();
    file.write_all(&plot_gp_data.into_bytes()).unwrap();

    println!("Plot data written to plot.gp! Invoking gnuplot...");
    println!("{:#?}",
             Command::new("gnuplot").args(["plot.gp"]).output().unwrap());
}
