use std::io::{BufReader, BufWriter, Read, Write};
mod time_guard;
use parser::parse_gpx;
use time_guard::clean_stream;
mod dissim;
mod parser;
mod tradis;
fn main() {
    let metric = std::env::args().nth(1).expect("no metric given!");
    let input_path = std::env::args().nth(2).expect("no input dir given!");
    let output_file = std::env::args().nth(3).expect("no output filename given!");
    match metric.as_ref() {
        "d" => {
            apply_metric(input_path, output_file, &dissim::similarity);
        }
        "t" => {
            apply_metric(input_path, output_file, &tradis::similarity);
        }
        "b" => {
            todo!("both")
        }
        _ => {
            panic!("Choose a valid metric to use as first argument: (d)issim, (t)radis, or (b)oth.")
        }
    }
}

fn apply_metric(
    input_path: String,
    output_file: String,
    metric: &dyn Fn(&[[f64; 3]], &[[f64; 3]]) -> f64,
) {
    let paths = std::fs::read_dir(input_path).expect("invalid input directory.");
    let mut trjs: Vec<Vec<[f64; 3]>> = vec![];
    let mut filenames: Vec<String> = vec![];
    for path in paths {
        let path = path.unwrap();
        let filename = std::fs::File::open(path.path());
        let mut reader = BufReader::new(filename.unwrap());
        let mut contents: String = String::new();
        reader.read_to_string(&mut contents).unwrap();
        let trj = parse_gpx(contents);
        let trj = clean_stream(trj);
        trjs.push(trj);
        let filename = path
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        filenames.push(filename);
    }
    let output_file = std::fs::File::create(output_file).expect("Could not create file.");
    let mut results: BufWriter<std::fs::File> = BufWriter::new(output_file);
    for (i, trj_a) in trjs.iter().enumerate() {
        for (j, trj_b) in trjs.iter().enumerate() {
            if i < j {
                if let Some((start, end)) = get_common_time_span(trj_a, trj_b) {
                    let trj_a = trim(trj_a, &start, &end);
                    let trj_b = trim(trj_b, &start, &end);
                    let trj_a = align_time_to_zero(trj_a);
                    let trj_b = align_time_to_zero(trj_b);
                    if trj_a.len() < 2 || trj_b.len() < 2 {
                        write_no_result(
                            &mut results,
                            filenames[i].to_string(),
                            filenames[j].to_string(),
                        );
                        continue;
                    }
                    let now = std::time::Instant::now();
                    let result = metric(&trj_a, &trj_b);
                    let dt = now.elapsed().as_micros();
                    writeln!(
                        results,
                        "{},{},{},{}",
                        filenames[i], filenames[j], result, dt
                    )
                    .expect("Could not write to file.");
                } else {
                    write_no_result(
                        &mut results,
                        filenames[i].to_string(),
                        filenames[j].to_string(),
                    );
                }
            }
        }
    }
}

fn write_no_result(results: &mut BufWriter<std::fs::File>, name_a: String, name_b: String) {
    writeln!(results, "{},{},{},{}", name_a, name_b, f64::NAN, f64::NAN)
        .expect("Could not write to file.");
}

pub fn get_common_time_span(trj_a: &[[f64; 3]], trj_b: &[[f64; 3]]) -> Option<(f64, f64)> {
    let a_len = trj_a.len() - 1;
    let b_len = trj_b.len() - 1;
    let start: f64 = if trj_a[0][2] > trj_b[0][2] {
        trj_a[0][2]
    } else {
        trj_b[0][2]
    };
    let end: f64 = if trj_a[a_len][2] < trj_b[b_len][2] {
        trj_a[a_len][2]
    } else {
        trj_b[b_len][2]
    };
    if end - start < 0.0 {
        None
    } else {
        Some((start, end))
    }
}

pub fn interpolate(t: &f64, p: &[f64; 3], q: &[f64; 3]) -> [f64; 3] {
    assert!((p[2]..=q[2]).contains(t));
    let dx = q[0] - p[0];
    let dy = q[1] - p[1];
    let s = (t - p[2]) / (q[2] - p[2]);
    [p[0] + dx * s, p[1] + dy * s, *t]
}

pub fn trim(trj: &[[f64; 3]], start: &f64, end: &f64) -> Vec<[f64; 3]> {
    // Adjust the trajectory to start and end at the given times
    let mut start_idx: usize = 0;
    let mut end_idx: usize = trj.len() - 1;
    let mut new_start: [f64; 3] = trj[start_idx];
    let mut new_end: [f64; 3] = trj[end_idx];
    for i in 0..(trj.len() - 1) {
        let point_p: [f64; 3] = trj[i];
        let point_q: [f64; 3] = trj[i + 1];
        if (point_p[2]..point_q[2]).contains(start) {
            new_start = interpolate(start, &point_p, &point_q);
            start_idx = i;
        };
        if (point_p[2]..point_q[2]).contains(end) {
            new_end = interpolate(end, &point_p, &point_q);
            end_idx = i + 1;
        };
    }
    let mut coords: Vec<[f64; 3]> = trj[start_idx..(end_idx + 1)].to_vec();
    coords[0] = new_start;
    coords[end_idx - start_idx] = new_end;
    coords
}

pub fn align_time_to_zero(trj: Vec<[f64; 3]>) -> Vec<[f64; 3]> {
    let offset: f64 = trj[0][2];
    trj.into_iter()
        .map(|point| [point[0], point[1], point[2] - offset])
        .collect::<Vec<[f64; 3]>>()
}
