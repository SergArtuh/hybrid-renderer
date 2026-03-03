// mod assets;
// mod core;
// mod examples;
// mod renderer;
// mod stage;
// mod util;
// use std::env;

// fn main() {
//     env_logger::init();
//     let args: Vec<String> = env::args().collect();
//     if args.len() < 2 {
//         println!("Usage: cargo run --bin wgpu-examples <example_name>");
//         println!("Available examples:");
//         println!("  simple_triangle");
//         println!("  texture");
//         println!("  primitive_geometry");
//         return;
//     }

//     match args[1].as_str() {
//         "simple_triangle" => examples::simple_triangle::run(),
//         "texture" => examples::texture::run(),
//         "primitive_geometry" => examples::primitive_geometry::run(),
//         _ => println!("Unknown example: {}", args[1]),
//     }
// }
