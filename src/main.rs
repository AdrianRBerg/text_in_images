use image::io::Reader as ImageReader;
use image::{GenericImage, GenericImageView};
use native_dialog::FileDialog;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;
use std::{thread, time};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    let path = FileDialog::new()
        .set_location("~/Desktop")
        .add_filter("PNG Image", &["png"])
        .add_filter("JPEG Image", &["jpg", "jpeg"])
        .show_open_single_file()
        .unwrap();
    println!("{:?}", path);
    match path {
        Some(path) => add_text_to_image(path),
        None => process::exit(1),
    };
}

fn add_text_to_image(path: PathBuf) {
    let binary_vector = get_binary_vector();
    let mut sliced_vector: Vec<u8> = Vec::with_capacity(binary_vector.len() * 2);
    println!("{}", binary_vector.len());
    let time1 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    for byte in binary_vector {
        // Get the 4 most significant bits
        sliced_vector.push((byte & 0xF0) >> 4);
        // Shift to the left to get least significant bits in the most significant part
        sliced_vector.push(byte & 0x0F);
    }
    let time2 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    println!("Time taken: {}", time2 - time1);
    println!("Starting to read image. ");
    let time3 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    let img = ImageReader::open(path).unwrap().decode().unwrap();
    let time4 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    println!("Read image into file, cloning...");
    let mut new_img = img.clone();
    let time5 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    println!("Cloned. \n");
    println!("Time taken to load image: {} ms\nTime taken to clone image: {}", time4 - time3, time5 - time4);
    let dimensions = img.dimensions();
    if (dimensions.0 * dimensions.1) < sliced_vector.len() as u32 {
        println!("Too much text for the given image.");
        process::exit(1);
    }
    println!("Starting to embed the text into the image");
    println!("{}", sliced_vector.len() % 3);
    match sliced_vector.len() {
        1 => sliced_vector.extend_from_slice(&[0, 0]),
        2 => sliced_vector.push(0),
        _ => ()
    }
    let bytes_iter = sliced_vector.chunks(3);
    for (mut pixel, chunk) in img.pixels().zip(bytes_iter) {
        let mut rgba = pixel.2; // Grabs the RGBA thingy
        rgba[0] = (rgba[0] & 0xF0) + chunk[0];
        rgba[1] = (rgba[1] & 0xF0) + chunk[1];
        rgba[2] = (rgba[2] & 0xF0) + chunk[2];
        new_img.put_pixel(pixel.0, pixel.1, rgba);
    }
    new_img.save("finished.png").unwrap();
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
                .set_location("~/Desktop")
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
