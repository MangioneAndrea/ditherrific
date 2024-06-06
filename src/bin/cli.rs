use clap::Parser;

use ditherrific;
use image::DynamicImage;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum, default_value_t = ditherrific::algorithms::Options::FloydSteinberg)]
    algorithm: ditherrific::algorithms::Options,

    #[arg(short, long)]
    output: Option<clio::OutputPath>,

    #[arg(short, long)]
    input: clio::InputPath,
}

fn main() {
    let args: Args = Args::parse();

    let folder = args.input.path().with_file_name(format!(
        "{}_output.png",
        args.input.path().file_stem().unwrap().to_str().unwrap(),
    ));

    let output = args
        .output
        .map(|o| o.path().to_path_buf())
        .unwrap_or(folder);

    let img: DynamicImage = image::io::Reader::open(args.input.path().path())
        .unwrap()
        .decode()
        .unwrap();

    let img: DynamicImage = ditherrific::transform(img.into(), args.algorithm).into();

    img.into_luma8().save(output.as_path()).unwrap();
}
