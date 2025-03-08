use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use clap::Parser;
use pngme::{
    Png,
    chunk::PngChunk,
    chunk_type::{PngChunkType, PngChunkTypeBinaryData},
};

/// PNG message encoder
#[derive(Parser, Debug)]
#[command(name = "pngme")]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Encodes a new message in a PNG file under a chunk with a certain type
    Encode {
        /// A path to the PNG file
        #[arg(required = true)]
        file_path: String,

        /// The chunk type under which the message will be encoded
        #[arg(required = true)]
        chunk_type: String,

        /// The message
        #[arg(required = true)]
        message: String,

        /// Alternative path to write modified content
        #[arg(required = false)]
        output_file_path: Option<String>,
    },
    /// Decodes a possibly existing message in a PNG file under a chunk with a certain type
    Decode {
        /// A path to the PNG file
        #[arg(required = true)]
        file_path: String,

        /// The chunk type under which the message should have been encoded
        #[arg(required = true)]
        chunk_type: String,
    },
    /// Removes a chunk with a certain type from a PNG file (useful when you want to remove a message)
    Remove {
        /// A path to the PNG file
        #[arg(required = true)]
        file_path: String,

        /// The chunk type under which the message should have been encoded
        #[arg(required = true)]
        chunk_type: String,
    },
    /// Displays the contents of a PNG file (useful for debugging)
    Print {
        /// A path to the PNG file
        #[arg(required = true)]
        file_path: String,
    },
}

fn open_file_for_rewrite(file_path: &String) -> anyhow::Result<File> {
    Ok(OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?)
}

fn create_png_from_file_bytes(file: &mut File) -> anyhow::Result<Png> {
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    let png = Png::try_from(&content as &[u8])?;

    Ok(png)
}

fn handle_encode_command(
    file_path: String,
    chunk_type: String,
    message: String,
    output_file_path: Option<String>,
) -> anyhow::Result<()> {
    let mut file = File::open(&file_path)?;
    let mut png = create_png_from_file_bytes(&mut file)?;

    png.remove_first_chunk(&chunk_type).ok();

    let chunk_type_bytes: PngChunkTypeBinaryData = chunk_type.as_bytes().try_into().unwrap();
    png.append_chunk(PngChunk::new(
        PngChunkType::try_from(chunk_type_bytes)?,
        message.as_bytes().to_vec(),
    ));

    let (mut write_target_file, write_target_file_path) = match output_file_path {
        Some(alt_path) => (open_file_for_rewrite(&alt_path)?, alt_path),
        None => (open_file_for_rewrite(&file_path)?, file_path),
    };

    write_target_file.write_all(&png.as_bytes())?;

    println!("new content written to file {write_target_file_path}");

    Ok(())
}

fn handle_decode_command(file_path: String, chunk_type: String) -> anyhow::Result<()> {
    let mut file = File::open(file_path)?;
    let png = create_png_from_file_bytes(&mut file)?;

    match png.chunk_by_type(&chunk_type) {
        Some(chunk) => match chunk.data_as_string() {
            Ok(message) => println!("{message}"),
            Err(_) => println!(
                "data in this chunk is not compatible with utf-8 encoding, most likely there is no encoded message here"
            ),
        },
        None => println!("couldn't find a chunk with this type"),
    };

    Ok(())
}

fn handle_remove_command(file_path: String, chunk_type: String) -> anyhow::Result<()> {
    let mut file = File::open(&file_path)?;
    let mut png = create_png_from_file_bytes(&mut file)?;

    if let Err(_) = png.remove_first_chunk(&chunk_type) {
        println!("couldn't find a chunk with this type");
        return Ok(());
    };

    let mut file = open_file_for_rewrite(&file_path)?;
    file.write_all(&png.as_bytes())?;

    println!("removed");

    Ok(())
}

fn handle_print_command(file_path: String) -> anyhow::Result<()> {
    let mut file = File::open(file_path)?;
    let png = create_png_from_file_bytes(&mut file)?;

    println!("{}", png);

    Ok(())
}

fn main() {
    let args = Cli::parse();

    match args.commands {
        Commands::Encode {
            file_path,
            chunk_type,
            message,
            output_file_path,
        } => handle_encode_command(file_path, chunk_type, message, output_file_path),
        Commands::Decode {
            file_path,
            chunk_type,
        } => handle_decode_command(file_path, chunk_type),
        Commands::Remove {
            file_path,
            chunk_type,
        } => handle_remove_command(file_path, chunk_type),
        Commands::Print { file_path } => handle_print_command(file_path),
    }
    .unwrap();
}
