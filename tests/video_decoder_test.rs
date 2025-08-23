use aether_core::engine::{video_decoder::{VideoDecoder, VideoDecoderConfig, create_default_decoder, get_media_info}, VideoFormat};
use std::path::Path;
use std::env;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Get video path from command line arguments or use a default
    let args: Vec<String> = env::args().collect();
    let video_path = if args.len() > 1 {
        &args[1]
    } else {
        println!("Usage: {} <path_to_video_file>", args[0]);
        return Ok(());
    };
    
    println!("Testing video decoder with file: {}", video_path);
    
    // Test 1: Get media info
    println!("\n=== Test 1: Get Media Info ===");
    match get_media_info(video_path) {
        Ok(info) => {
            println!("Media info retrieved successfully:");
            println!("  Duration: {:.2} seconds", info.duration);
            println!("  Video streams: {}", info.video_streams.len());
            println!("  Audio streams: {}", info.audio_streams.len());
            
            if !info.video_streams.is_empty() {
                let stream = &info.video_streams[0];
                println!("  First video stream:");
                println!("    Index: {}", stream.index);
                println!("    Width: {}", stream.width);
                println!("    Height: {}", stream.height);
                println!("    Frame rate: {:.2} fps", stream.frame_rate);
                println!("    Codec: {}", stream.codec_name);
            }
        },
        Err(e) => {
            println!("Failed to get media info: {:?}", e);
            return Ok(());
        }
    }
    
    // Test 2: Decode frames
    println!("\n=== Test 2: Decode Frames ===");
    let mut config = VideoDecoderConfig::default();
    config.target_format = VideoFormat::RGB24;
    
    let mut decoder = VideoDecoder::new(config);
    match decoder.open(video_path) {
        Ok(_) => println!("Decoder opened successfully"),
        Err(e) => {
            println!("Failed to open decoder: {:?}", e);
            return Ok(());
        }
    }
    
    // Decode and measure 100 frames or until EOF
    let start_time = Instant::now();
    let mut frame_count = 0;
    let max_frames = 100;
    
    println!("Decoding {} frames...", max_frames);
    
    while frame_count < max_frames {
        match decoder.decode_video_frame() {
            Ok(frame) => {
                if frame_count % 10 == 0 {
                    println!("  Frame {}: PTS: {}, Size: {}x{}, Format: {:?}",
                        frame_count, frame.pts, frame.width, frame.height, frame.format);
                }
                frame_count += 1;
            },
            Err(e) => {
                println!("  Decoding stopped: {:?}", e);
                break;
            }
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Decoded {} frames in {:.2?}", frame_count, elapsed);
    println!("Average FPS: {:.2}", frame_count as f64 / elapsed.as_secs_f64());
    
    // Test 3: Seeking
    println!("\n=== Test 3: Seeking ===");
    let media_info = decoder.get_media_info().unwrap();
    let seek_positions = [
        0.0,
        media_info.duration * 0.25,
        media_info.duration * 0.5,
        media_info.duration * 0.75,
    ];
    
    for &pos in &seek_positions {
        println!("  Seeking to {:.2} seconds...", pos);
        match decoder.seek(pos) {
            Ok(_) => {
                match decoder.decode_video_frame() {
                    Ok(frame) => {
                        println!("    Decoded frame at PTS: {}, Position: {:.2} seconds",
                            frame.pts, decoder.get_position());
                    },
                    Err(e) => println!("    Failed to decode frame after seek: {:?}", e),
                }
            },
            Err(e) => println!("    Failed to seek: {:?}", e),
        }
    }
    
    // Test 4: Stream selection (if multiple streams available)
    if media_info.video_streams.len() > 1 {
        println!("\n=== Test 4: Stream Selection ===");
        for stream in &media_info.video_streams {
            println!("  Selecting video stream {}: {}x{} ({})", 
                stream.index, stream.width, stream.height, stream.codec_name);
            
            match decoder.select_video_stream(stream.index) {
                Ok(_) => {
                    match decoder.decode_video_frame() {
                        Ok(frame) => {
                            println!("    Decoded frame from stream {}: {}x{}", 
                                stream.index, frame.width, frame.height);
                        },
                        Err(e) => println!("    Failed to decode frame from stream {}: {:?}", stream.index, e),
                    }
                },
                Err(e) => println!("    Failed to select stream {}: {:?}", stream.index, e),
            }
        }
    }
    
    // Close the decoder
    println!("\n=== Closing Decoder ===");
    match decoder.close() {
        Ok(_) => println!("Decoder closed successfully"),
        Err(e) => println!("Failed to close decoder: {:?}", e),
    }
    
    println!("\nAll tests completed!");
    Ok(())
}
