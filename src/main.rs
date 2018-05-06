#![allow(dead_code)]

extern crate wiringpi;

mod utility;

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::process::Command;

use utility::*;

fn main() {
    // Load lookup data
    let mut file = File::open("calibrations.txt").expect("no calibrations file");
    let mut cal_str = String::new();
    file.read_to_string(&mut cal_str)
        .expect("Unable to read calbrations to string");
    // Creat lookup table
    let mut lookup = LookupTable::new(2592);
    let mut words = cal_str.split_whitespace();

    // Insert lookup values and fill
    while let Some(pos_str) = words.next() {
        lookup.add_exact(
            pos_str.parse().expect("unable to parse pos_str"),
            words
                .next()
                .expect("odd number of words")
                .parse()
                .expect("unable to parse dist_str"),
        );
    }
    lookup.fill();
    // println!("{:?}", lookup);

    let args: Vec<String> = env::args().collect();

    // Define the placement and height of the scan line
    let scan_line_top: u32 = if args.len() >= 2 {
        args[1].parse::<u32>().unwrap()
    } else {
        1028
    };
    let scan_line_height: u32 = if args.len() >= 3 {
        args[2].parse::<u32>().unwrap()
    } else {
        5
    };

    // Defines the width of a bucket
    let bucket_width: u32 = if args.len() >= 4 {
        args[3].parse::<u32>().unwrap()
    } else {
        20
    };

    // Initialize image
    let mut scan_line_image = vec![0u8; 3 * (2592 * scan_line_height) as usize];

    // Initialize dot position
    let mut dot_position: Option<u32>;

    // Initialize gpio
    let pi = wiringpi::setup();
    let pin = pi.pwm_pin();

    loop {
        // Take picture
        Command::new("raspistill")
            .args(&["-o", "pic.bmp", "--nopreview", "-t", "10", "-e", "bmp"])
            .output()
            .expect("Unable to exectue take picture command.");
        println!("took picture");

        // Load the image file
        let mut image_file = File::open("pic.bmp").expect("Unable to openimage file.");
        // let mut image_file = File::open(&args[1]).expect("Unable to openimage file.");

        // Read image header into buffer
        let mut header_buffer = [0u8; 54];
        image_file
            .read_exact(&mut header_buffer)
            .expect("Unable to read image header");

        // Get the image width and height from the header
        let (width, height) = (
            header_buffer[19] as u32 * 256 + header_buffer[18] as u32,
            header_buffer[23] as u32 * 256 + header_buffer[22] as u32,
        );

        // Read image into image buffer
        image_file
            .seek(SeekFrom::Start(
                (54 + (height - scan_line_top - scan_line_height) * width * 3) as u64,
            ))
            .expect("Unable to seek to scan line top in image file");
        image_file
            .read_exact(&mut scan_line_image)
            .expect("Unable to read image scan line.");
        println!("read image into buffer");

        // Run the algorithm

        // Initialize buckets and avg_pixels
        let mut buckets: HashMap<u32, Bucket> = HashMap::new();
        let mut avg_pixels = vec![(0u32, 0u32, 0u32); width as usize];

        // Get average pixels and add them to buckets
        for i in 0..width {
            for j in 0..scan_line_height {
                let color = bmp_pixel(&scan_line_image, width, i, j);
                avg_pixels[i as usize].0 += color.0;
                avg_pixels[i as usize].1 += color.1;
                avg_pixels[i as usize].2 += color.2;
            }
            avg_pixels[i as usize].0 /= scan_line_height;
            avg_pixels[i as usize].1 /= scan_line_height;
            avg_pixels[i as usize].2 /= scan_line_height;
            let bucket = i / bucket_width;
            buckets.entry(bucket).or_insert_with(Bucket::new).insert(
                i as u32 / bucket_width,
                i,
                SimpleColor::from_color(avg_pixels[i as usize]),
            );
        }

        // Amalgomate Buckets
        let mut done = false;
        while !done {
            let mut keys_to_merge: Vec<(u32, u32)> = Vec::new();

            // For each pair of different buckets, check if they are adjacent.
            // If they are, add the pair of their keys to the keys_to_merge
            for (a_key, a_bucket) in &buckets {
                for (b_key, b_bucket) in &buckets {
                    if a_key != b_key && a_bucket.adjacent(b_bucket) {
                        keys_to_merge.push((*a_key, *b_key));
                    }
                }
            }

            // For each pair of keys to merge, merge the corresponding buckets
            for pair in &keys_to_merge {
                // HashMap::remove() on the buckets doesn't always work
                // because a bucket may have already been removed and merged with another.
                // To account for this, we have to check.
                let a_opt = buckets.remove(&pair.0);
                let b_opt = buckets.remove(&pair.1);
                if let (Some(mut a), Some(mut b)) = (a_opt.clone(), b_opt.clone()) {
                    let new_bucket = a.merge(&mut b);
                    let new_key = new_bucket.main_key();
                    buckets.insert(new_key, new_bucket);
                } else if let Some(mut a) = a_opt {
                    let new_key = a.main_key();
                    buckets.insert(new_key, a);
                } else if let Some(mut b) = b_opt {
                    let new_key = b.main_key();
                    buckets.insert(new_key, b);
                }
            }
            // We are done amalgomating buckets when there are no more keys to marge.
            done = keys_to_merge.is_empty();
        }

        // Determine dot positions
        let mut final_color_positions: BTreeMap<u32, SimpleColor> = BTreeMap::new();
        for (_key, bucket) in &buckets {
            let mut pos_sum = 0f32;
            for &pixel in &bucket.points {
                pos_sum += pixel as f32;
            }
            let pos_result = pos_sum / bucket.points.len() as f32;
            final_color_positions.insert(pos_result as u32, bucket.simple_color);
        }

        println!();
        for fcp in &final_color_positions {
            println!("{:?} at {}", fcp.1, fcp.0)
        }
        println!();

        let final_color_positions = final_color_positions
            .iter()
            .map(|pair| (*pair.0, *pair.1))
            .collect::<Vec<(u32, SimpleColor)>>();

        // find the dot
        dot_position = None;

        for (i, &(pos, color)) in final_color_positions.iter().enumerate() {
            use SimpleColor::*;
            if color == White
                && ((i == 0 || final_color_positions[i - 1].1 == Red)
                    || (i == final_color_positions.len() - 1
                        || final_color_positions[i + 1].1 == Red))
            {
                dot_position = Some(pos);
                break;
            }
        }

        // if the dot was not found looking for red white red, look for just red
        if dot_position.is_none() {
            for &(pos, color) in final_color_positions.iter() {
                if color == SimpleColor::Red {
                    dot_position = Some(pos);
                    break;
                }
            }
        }
        // if the dot was still not found looking for red, look for white
        if dot_position.is_none() {
            for &(pos, color) in final_color_positions.iter() {
                if color == SimpleColor::White {
                    dot_position = Some(pos);
                    break;
                }
            }
        }

        match dot_position {
            Some(pos) => {
                println!("Dot found at x = {}", pos);
                let final_pos = lookup.dist(pos as usize);
                println!("The dot is {} inches away", final_pos);
                pin.write((final_pos.powf(0.33333) * 149.12) as u16);
            }
            None => {
                println!("Dot not found!");
                pin.write(0);
            }
        }
    }
}
