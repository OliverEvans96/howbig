use std::path::{Path, PathBuf};

use anyhow::anyhow;
use clap::Parser;
use tiny_skia::Pixmap;
use usvg::{fontdb, Tree};

/// Count the number of opaque pixels in an SVG.
#[derive(Debug, Parser)]
struct Opts {
    /// Path to SVG file
    path: PathBuf,
    /// Calculate opaque as a percentage
    #[clap(short, long)]
    percentage: bool,
    /// Calculate opaque as a percentage of bounding square
    #[clap(short, long)]
    square: bool,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let svg = load_svg(&opts.path)?;
    let pixmap = render_svg(svg)?;
    let area = calculate_area(&pixmap);

    if opts.percentage {
        let w = pixmap.width();
        let h = pixmap.height();
        let total_area = if opts.square {
            let longest_dim = w.max(h);
            longest_dim * longest_dim
        } else {
            w * h
        };
        let ratio = (area as f64) / (total_area as f64);
        let percentage = 100.0 * ratio;
        println!("{:.0}%", percentage);
    } else {
        println!("{}", area);
    }

    Ok(())
}

fn load_svg(path: &Path) -> anyhow::Result<Tree> {
    let mut opt = usvg::Options::default();
    // Get file's absolute directory.
    opt.resources_dir = std::fs::canonicalize(path)
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    let svg_data = std::fs::read(path)?;
    let tree = usvg::Tree::from_data(&svg_data, &opt, &fontdb)?;

    Ok(tree)
}

fn render_svg(svg: Tree) -> anyhow::Result<Pixmap> {
    let pixmap_size = svg.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or(anyhow!("zero size pixmap"))?;
    resvg::render(&svg, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    Ok(pixmap)
}

fn calculate_area(pixmap: &Pixmap) -> u32 {
    let mut area = 0;
    for pixel in pixmap.pixels() {
        if pixel.is_opaque() {
            area += 1;
        }
    }

    return area;
}
