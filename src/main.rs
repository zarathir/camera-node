use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::Device;
use v4l::FourCC;
use zenoh::prelude::r#async::AsyncResolve;

fn main() {
    let mut dev = Device::new(0).expect("Failed to open device");

    let mut fmt = dev.format().expect("Failed to read format");
    fmt.width = 1280;
    fmt.height = 720;
    fmt.fourcc = FourCC::new(b"MJPG");
    dev.set_format(&fmt).expect("Failed to write format");

    println!("Format in use:\n{}", fmt);

    let mut stream = Stream::with_buffers(&mut dev, Type::VideoCapture, 4)
        .expect("Failed to create buffer stream");

    async_std::task::block_on(async move {
        let config = zenoh::config::peer();
        let session = zenoh::open(config).res_async().await.unwrap();

        let publisher = session
            .declare_publisher("camera_node")
            .res_async()
            .await
            .unwrap();

        loop {
            let buf = stream.next().unwrap().0;
            publisher.put(buf).res().await.unwrap();
        }
    });
}
