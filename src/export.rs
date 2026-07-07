use crate::capture::{Marker, Sample, CHANNEL_NAMES};
use chrono::Local;

pub fn to_csv(samples: &[Sample], markers: &[Marker]) -> String {
    let mut out = format!("elapsed_s,{}\n", CHANNEL_NAMES.join(","));

    for s in samples {
        let cols: Vec<String> = s.temps.iter()
            .map(|&v| if v.is_nan() { String::new() } else { format!("{:.1}", v) })
            .collect();
        out.push_str(&format!("{:.3},{}\n", s.elapsed, cols.join(",")));
    }

    if !markers.is_empty() {
        out.push_str("\n# Markers\n# elapsed_s,label\n");
        for m in markers {
            out.push_str(&format!("# {:.3},{}\n", m.elapsed, m.label));
        }
    }

    out
}

pub fn write(samples: &[Sample], markers: &[Marker]) -> std::io::Result<String> {
    let filename = format!("thermode_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
    std::fs::write(&filename, to_csv(samples, markers))?;
    Ok(filename)
}
