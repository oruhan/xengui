// build.rs
use std::fs::File;
use std::path::Path;
use ico::IconDir;
use image::io::Reader as ImageReader;
use image::imageops::FilterType;

fn main() {
    // Rerun the build if icon.png changes
    println!("cargo:rerun-if-changed=assets/icon.png");

    let input_path = "assets/icon.png";
    let output_path = "assets/icon.ico";

    if Path::new(input_path).exists() {
        // Read the PNG and convert it to RGBA
        let img = ImageReader::open(input_path).unwrap().decode().unwrap();
        let rgba = img.to_rgba8();

        let mut icon_dir = IconDir::new(ico::ResourceType::Icon);

        // Required sizes for Windows icons
        let sizes = [16, 32, 48, 64, 128, 256];
        for size in sizes {
            let resized = image::imageops::resize(&rgba, size, size, FilterType::Lanczos3);
            let icon_image = ico::IconImage::from_rgba_data(size, size, resized.to_vec());
            icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image).unwrap());
        }

        // Create the .ico file
        let out_file = File::create(output_path).unwrap();
        icon_dir.write(out_file).unwrap();
    }

    // Set the exe icon on Windows
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();
        res.set_icon(output_path);
        res.compile().unwrap();
    }
}
