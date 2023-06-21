mod huffman;

use std::{
    io::Error,
    time::{Duration, Instant},
};

fn print_help() {
    println!("usage: huffman [c]ompress|[d]ecompress <in_file> <out_file>")
}

fn print_duration(duration: Duration) {
    if duration.as_secs() > 0 {
        println!("Time: {:.1}s", duration.as_secs_f32());
    } else {
        println!("Time: {}ms", duration.as_millis());
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        print_help();
        return Ok(());
    }

    let in_file = &args[2];
    let out_file = &args[3];

    if args[1].starts_with("c") {
        println!("Compressing '{}' in '{}'...", in_file, out_file);

        let start = Instant::now();
        huffman::encode_file(in_file.clone(), out_file.clone())?;
        let duration = start.elapsed();

        println!(
            "'{}' was compressed successfully in '{}'",
            in_file, out_file
        );
        print_duration(duration);
    } else if args[1].starts_with("d") {
        println!("Decompressing '{}' in '{}'...", in_file, out_file);

        let start = Instant::now();
        huffman::decode_file(in_file.clone(), out_file.clone())?;
        let duration = start.elapsed();

        println!(
            "'{}' was decompressed successfully in '{}'",
            in_file, out_file
        );
        print_duration(duration);
    } else {
        print_help();
    }

    Ok(())
}
