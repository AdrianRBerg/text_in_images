use native_dialog::{FileDialog};
use std::process;
use std::path::PathBuf;
use std::io;
use std::fs;
use std::io::Cursor;
use image::GenericImageView;
use image::io::Reader as ImageReader;

fn main() {

    let path = FileDialog::new()
        .set_location("~/Desktop")
        .add_filter("PNG Image", &["png"])
        .add_filter("JPEG Image", &["jpg", "jpeg"])
        .show_open_single_file()
        .unwrap();

    match path {
        Some(path) => add_text_to_image(path),
        None => process::exit(1),
    };
}

fn add_text_to_image(path: PathBuf) {
    println!("Original binary:");
    let text = get_text_input();
    for el in &text {
        print!("{:08b} ", el);
        print!("         ");
    };

    let mut sliced_vector: Vec<u8> = Vec::new();
    for byte in text {
        sliced_vector.push(byte & 0xF0);
        let mut x = byte & 0x0F;
        // Shift 4 spots ahead so we can concern ourselves only with grabbing the most significant
        // bits when we add text to our image.
        x = x << 4;
        sliced_vector.push(x);
    }
    println!("Sliced binary:");
    for el in &sliced_vector {
        print!("{:08b} ", el);
    };
    // let img = ImageReader::open(path).unwrap().decode().unwrap();
    //
    // for x in img.pixels() {
    //     println!("{:?}", x);
    // }
}

/// Get text input from the user, either by file or console.
fn get_text_input() -> Vec<u8> {
    println!("Choose between the following options:\n1) Add text by console input 2) Add text by text file");
    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read input. Exiting program");
    choice.pop();
    let text = match choice.as_str() {
        // Clean and simple
        "1" => {
            println!("Type the text you want to embed into the image.\n");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input. Exiting program");
            input
        },
        // Using a text file rather than console allows for more freeform, such as newlines and etc.
        // if the shell/terminal doesn't allow for that
        "2" => {
            println!("Select a text file");
            let path = FileDialog::new()
                .set_location("~/Desktop")
                .show_open_single_file()
                .unwrap()
                .expect("Not a valid path. Exiting program");
            let input = fs::read_to_string(path)
                .expect("Should have been able to read the file");
            input
        },
        _ => {
            println!("Invalid option. Exiting program");
            process::exit(1);
        }
    };

    let mut text_bytes = text.into_bytes();
    text_bytes.push(0b00000000);
    text_bytes

}