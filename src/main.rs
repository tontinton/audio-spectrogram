use clap::Parser;
use image::{Rgb, RgbImage};
use rustfft::{
    num_complex::{Complex, ComplexFloat},
    FftPlanner,
};

const CHUNK_SIZE: usize = 4096;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Transform mp3 audio to a spectogram of frequencies using FFT.
struct Args {
    /// The input mp3 file path.
    input_file: String,

    #[arg(default_value_t=String::from("output.png"))]
    /// Output image file path.
    output_file: String,
}

fn main() {
    let args = Args::parse();

    let data = std::fs::read(args.input_file).expect("Could not open file");
    let (_header, samples) = puremp3::read_mp3(&data[..]).expect("Invalid MP3");

    let mut chunks: Vec<Vec<Complex<f32>>> = Vec::new();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(CHUNK_SIZE);

    for (offset, (left, _)) in samples.enumerate() {
        if offset % CHUNK_SIZE == 0 {
            chunks.push(vec![Complex { re: 0.0, im: 0.0 }; CHUNK_SIZE]);
        }

        let len = chunks.len();
        let chunk = &mut chunks[len - 1];

        chunk[offset % CHUNK_SIZE].re = left;
        chunk[offset % CHUNK_SIZE].im = 0.0;

        if (offset + 1) % CHUNK_SIZE == 0 {
            fft.process(chunk);
        }
    }

    let mut image = RgbImage::new(chunks.len() as u32, (CHUNK_SIZE / 2) as u32);

    for (x, chunk) in chunks.into_iter().enumerate() {
        let chunk_half_len = chunk.len() / 2 - 1;
        for (y, num) in chunk.into_iter().enumerate() {
            // Second half is a mirror of first half.
            if y > chunk_half_len {
                break;
            }
            let val = (num.abs() + 1.0).log10();
            image.put_pixel(
                x as u32,
                (chunk_half_len - y) as u32,
                Rgb([(val * 100.0) as u8, 0, (val * 50.0) as u8]),
            );
        }
    }

    image.save(args.output_file).unwrap();
}
