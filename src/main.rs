use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

use image::{GenericImage, GenericImageView};
use image::io::Reader as ImageReader;
use native_dialog::FileDialog;

fn main() {
    println!("Choose an option:\n1) Read input into image\n2) Extract text from image");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input. Exiting program");
    input.pop();
    if input != "1" && input != "2" {
        println!("Invalid option. Exiting.");
        process::exit(1);
    };
    let path = FileDialog::new()
        .add_filter("PNG Image", &["png"])
        .add_filter("JPEG Image", &["jpg", "jpeg"])
        .show_open_single_file()
        .expect("Invalid path. Exiting program");

    let unwrapped_path = if path != None {
        path.expect("Unexpected error in path.")
    } else {
        println!("Invalid path. Exiting.");
        process::exit(1)
    };
    match input.as_str() {
        "1" => add_text_to_image(unwrapped_path),
        "2" => extract_text_from_image(unwrapped_path),
        _ => println!("This error shouldn't be possible."),
    }
}

fn extract_text_from_image(path: PathBuf) {
    let img = ImageReader::open(path).expect("Error in opening the image").decode().expect("Error in decoding the image");
    let mut binary_vector: Vec<u8> = Vec::new();

    for pixel in img.pixels() {
        let rgba = pixel.2;
        let r = rgba[0] & 0x0F;
        let g = rgba[1] & 0x0F;
        let b = rgba[2] & 0x0F;
        binary_vector.extend_from_slice(&[r, g, b]);
    }

    while (binary_vector.len() % 2) != 0 {
        binary_vector.push(0b00000000);
    }
    let mut assembled_vector: Vec<u8> = Vec::with_capacity(binary_vector.len() / 2);
    let bytes_iter = binary_vector.chunks(2);
    // Combines chunks of bytes into full bytes since
    // in this state they only exist in half_states
    for chunk in bytes_iter {
        let mut byte = chunk[0];
        byte = byte << 4;
        let mut part = chunk[1];
        part = part & 0x0F;
        byte = byte + part;
        assembled_vector.push(byte);
        if byte == 0b00000000 {
            break;
        }
    }

    // Remove lone 0b00000000
    assembled_vector.pop();
    let s = String::from_utf8_lossy(&assembled_vector);
    println!("{}", s);
}

fn add_text_to_image(path: PathBuf) {
    let binary_vector = get_binary_vector();
    let mut sliced_vector: Vec<u8> = Vec::with_capacity(binary_vector.len() * 2);
    for byte in binary_vector {
        // Get the 4 most significant bits and shift to the right
        sliced_vector.push((byte & 0xF0) >> 4);
        // Get the 4 least significant bits
        sliced_vector.push(byte & 0x0F);
    }
    println!("Starting to read image. ");
    let img = ImageReader::open(path).expect("Error in opening the image").decode().expect("Error in decoding the image");
    println!("Read image into file, cloning...");
    let mut new_img = img.clone();

    println!("Cloned. \n");
    let dimensions = img.dimensions();
    //
    if ((dimensions.0 * dimensions.1) as f32 * 1.5) < sliced_vector.len() as f32 {
        println!("Too much text for the given image.");
        process::exit(1);
    }
    println!("Starting to embed the text into the image");

    match sliced_vector.len() % 3 {
        1 => sliced_vector.extend_from_slice(&[0, 0]),
        2 => sliced_vector.push(0),
        _ => (),
    }

    let bytes_iter = sliced_vector.chunks(3);
    for (pixel, chunk) in img.pixels().zip(bytes_iter) {
        let mut rgba = pixel.2; // Grabs the RGBA thingy
        rgba[0] = (rgba[0] & 0xF0) + chunk[0];
        rgba[1] = (rgba[1] & 0xF0) + chunk[1];
        rgba[2] = (rgba[2] & 0xF0) + chunk[2];
        new_img.put_pixel(pixel.0, pixel.1, rgba);
    }
    new_img.save("finished.png").expect("Failed to save image");
    println!("Successfully saved image");
}

/// Get binary input from the user, either by file or text in console.
fn get_binary_vector() -> Vec<u8> {
    println!("Choose between the following options:\n1) Add text by console input 2) Add text by text file");
    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read input. Exiting program");
    choice.pop(); // Remove newline
    let mut bytes_vector = match choice.as_str() {
        // Clean and simple
        "1" => {
            println!("Type the text you want to embed into the image.\n");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input. Exiting program");
            let text_bytes = input.into_bytes();
            text_bytes
        }
        // Using a text file rather than console allows for more freeform, such as newlines and etc.
        // if the shell/terminal doesn't allow for that
        "2" => {
            println!("Select a text file");
            let path = FileDialog::new()
                .show_open_single_file()
                .unwrap()
                .expect("Not a valid path. Exiting program");
            let input = fs::read(path).expect("Should have been able to read the file");
            println!("WARNING: If the read file is not UTF-8 or ASCII compliant, this MAY fail.\n");
            println!("Successfully read file into vector. Size: {}", input.len());
            input
        }
        _ => {
            println!("Invalid option. Exiting program");
            process::exit(1);
        }
    };

    bytes_vector.push(0b00000000);
    bytes_vector
}
