use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

const DEFAULT_ASCII_WIDTH: u32 = 300;
const CHARACTER_ASPECT_RATIO: f64 = 0.5;
const ASCII_CHARS: &[u8] = b"@%#*+=-:,. ";

fn main() {
    if let Err(error) = run() {
        eprintln!("Input error: {error}");
    }
}

fn run() -> io::Result<()> {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush()?;
    println!("Welcome to asciiartinator!\nTurn images into ASCII text files.");

    loop {
        let choice =
            read_input("-------------------------------------\n1) Convert image\nq) Quit")?;
        match choice.trim() {
            "1" => convert_image()?,
            "q" | "Q" => return Ok(()),
            _ => {
                println!("Choose 1 or q.");
            }
        }
    }
}

fn convert_image() -> io::Result<()> {
    let invert = read_yes_no("Invert shades (dark mode)? [y/n]")?;

    let (image, image_path) = loop {
        let path = read_path("Enter the image path (Enter to return to menu):")?;
        if path.as_os_str().is_empty() {
            return Ok(());
        }

        match image::open(&path) {
            Ok(image) => {
                println!("\nImage found and loaded: {}", path.display());
                break (image, fs::canonicalize(&path)?);
            }
            Err(error) => println!("Could not load image: {error}\nTry another path."),
        }
    };

    let width = read_width()?;
    let ascii_art = image_to_ascii(&image, width, invert);

    loop {
        let output_path = read_path("Enter the output filename (Enter to cancel):")?;
        if output_path.as_os_str().is_empty() {
            return Ok(());
        }

        if output_path.exists() {
            if fs::canonicalize(&output_path)? == image_path {
                println!("Cannot overwrite the input image. Choose another output path.");
                continue;
            }
            if !output_path.is_file() {
                println!("The output path is not a file. Choose another path.");
                continue;
            }
            if !read_yes_no("This file already exists. Overwrite it? [y/n]")? {
                continue;
            }
        }

        match fs::write(&output_path, &ascii_art) {
            Ok(()) => {
                println!("\nASCII art saved to: {}", output_path.display());
                read_input("Press Enter to continue...")?;
                print!("\x1B[2J\x1B[H");
                io::stdout().flush()?;
                return Ok(());
            }
            Err(error) => println!("Could not save ASCII art: {error}\nTry another filename."),
        }
    }
}

fn image_to_ascii(image: &image::DynamicImage, width: u32, invert: bool) -> String {
    let grayscale = image.to_luma8();

    let height = (grayscale.height() as f64 / grayscale.width() as f64
        * width as f64
        * CHARACTER_ASPECT_RATIO)
        .round()
        .max(1.0) as u32;

    let resized = image::imageops::resize(
        &grayscale,
        width,
        height,
        image::imageops::FilterType::Triangle,
    );

    let mut ascii_art = String::new();

    let brightness_to_char = |brightness: u8| {
        let index = brightness as usize * (ASCII_CHARS.len() - 1) / 255;
        let index = if invert {
            ASCII_CHARS.len() - 1 - index
        } else {
            index
        };

        ASCII_CHARS[index] as char
    };

    for row in resized.rows() {
        for pixel in row {
            ascii_art.push(brightness_to_char(pixel[0]));
        }

        ascii_art.push('\n');
    }

    ascii_art
}

fn read_width() -> io::Result<u32> {
    loop {
        let input = read_input(&format!(
            "Output width in characters (1–1000, Enter for {DEFAULT_ASCII_WIDTH}):"
        ))?;
        if input.trim().is_empty() {
            return Ok(DEFAULT_ASCII_WIDTH);
        }
        match input.trim().parse::<u32>() {
            Ok(width) if (1..=1000).contains(&width) => return Ok(width),
            _ => println!("Enter a whole number between 1 and 1000."),
        }
    }
}

fn read_yes_no(prompt: &str) -> io::Result<bool> {
    loop {
        let answer = read_input(prompt)?;
        match answer.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please enter y or n."),
        }
    }
}

fn read_path(prompt: &str) -> io::Result<PathBuf> {
    let input = read_input(prompt)?;

    // Remove surrounding whitespace and one matching pair of double quotes.
    let path = input.trim();
    let path = path
        .strip_prefix('"')
        .and_then(|path| path.strip_suffix('"'))
        .unwrap_or(path);

    Ok(PathBuf::from(path))
}

fn read_input(prompt: &str) -> io::Result<String> {
    println!("\n{prompt}");
    let mut input = String::new();
    if io::stdin().read_line(&mut input)? == 0 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Input closed"));
    }
    Ok(input)
}
