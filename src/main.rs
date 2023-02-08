use std::io::Cursor;
use std::thread;
use std::time::Duration;

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GrayImage, RgbImage};
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};
use zenoh::prelude::sync::SyncResolve;

fn main() {
    let mut dev = Device::new(0).expect("Failed to open device");

    let mut fmt = dev.format().expect("Failed to read format");
    fmt.width = 1280;
    fmt.height = 720;
    fmt.fourcc = FourCC::new(b"RG10");
    dev.set_format(&fmt).expect("Failed to write format");

    println!("Format in use:\n{}", fmt);

    let mut stream = Stream::with_buffers(&mut dev, Type::VideoCapture, 4)
        .expect("Failed to create buffer stream");

    let config = zenoh::config::peer();
    let session = zenoh::open(config).res().unwrap();

    let publisher = session.declare_publisher("camera_node").res().unwrap();

    loop {
        let buf = stream.next().unwrap().0;
        let demosaiced = demosaic(buf, fmt.width as usize, fmt.height as usize);
        let image = DynamicImage::ImageLuma8(
            GrayImage::from_raw(fmt.width, fmt.height, demosaiced).unwrap(),
        );
        let color = image.into_rgb8();
        let mut encoded = vec![];
        JpegEncoder::new_with_quality(&mut encoded, 50)
            .encode(&color, fmt.width, fmt.height, image::ColorType::Rgb8)
            .unwrap();
        publisher.put(encoded).res().unwrap();
        thread::sleep(Duration::from_millis(50));
    }
}

fn demosaic(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut buf = vec![0; 6 * width * height];
    let mut dst = bayer::RasterMut::new(width, height, bayer::RasterDepth::Depth16, &mut buf);
    bayer::run_demosaic(
        &mut Cursor::new(&data[..]),
        bayer::BayerDepth::Depth16LE,
        bayer::CFA::RGGB,
        bayer::Demosaic::Linear,
        &mut dst,
    )
    .unwrap();

    buf
}
