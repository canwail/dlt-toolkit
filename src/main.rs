use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant
};

use dlt_parse::storage::{
    DltStorageWriter,
    DltStorageReader,
    StorageHeader
};

use structopt::StructOpt;

/// Expected command line arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "dlt-toolkit")]
struct CommandLineArguments {
    #[structopt(parse(from_os_str))]
    in_file: PathBuf,

    #[structopt(long)]
    max_size_mb: usize,
}

const MEGABYTE: usize = (1024 * 1024);

// Helper function for creating output files and writers
fn create_writer(base_name: &PathBuf, file_index: usize) -> Result<DltStorageWriter<BufWriter<File>>, std::io::Error> {
    let output_file_name = format!(
        "{}_{}.dlt",
        base_name.file_stem().unwrap_or_default().to_string_lossy(),
        file_index
    );
    let out_file = File::create(output_file_name)?;
    Ok(DltStorageWriter::new(BufWriter::new(out_file)))
}

fn main()  -> Result<(), Box<dyn std::error::Error>> {
    let args: CommandLineArguments = CommandLineArguments::from_args();

    // Derive the output file path
    let in_file = File::open(&args.in_file)?;
    let mut reader: DltStorageReader<BufReader<File>> = DltStorageReader::new(BufReader::new(in_file));

    let mut current_file_index = 0;
    let mut current_file_bytes_written = 0;
    let mut total_bytes_written = 0;

    let mut writer: DltStorageWriter<BufWriter<File>> = create_writer(&args.in_file, current_file_index)?;

    let start_time = Instant::now();

    while let Some(msg) = reader.next_packet() {
        let msg = msg?;
        let bytes_to_write = msg.packet.slice().len() + size_of::<StorageHeader>();
        let is_file_size_above_limit = (current_file_bytes_written + bytes_to_write) > args.max_size_mb * MEGABYTE;

        if is_file_size_above_limit {
            current_file_index += 1;
            current_file_bytes_written = 0;
            writer = create_writer(&args.in_file, current_file_index)?;

            let elapsed_time = start_time.elapsed();
            let total_mb_processed = (total_bytes_written / MEGABYTE) as f64;
            let mb_per_second = total_mb_processed / elapsed_time.as_secs_f64();
            println!("Processed {:.2} MB in @ ({:.2} MB/s)", total_mb_processed, mb_per_second);
        }

        writer.write_slice(msg.storage_header, msg.packet)?;

        current_file_bytes_written += bytes_to_write;
        total_bytes_written += bytes_to_write;

    }
    Ok(())
}
