use ffmpeg_next::{
    format::{input, Pixel},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    util::frame::video::Video,
};
use image::Luma;

fn main() -> Result<(), ffmpeg_next::Error> {
    ffmpeg_next::init().unwrap();

    let path = "/home/sebastian/Videos/Webcam/2021-07-14-140819.webm".to_string();
    if let Ok(mut ictx) = input(&path) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let mut decoder = input.codec().decoder().video()?;

        let dfmt = decoder.format();
        println!("dfmt {:?}", dfmt);
        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            // Pixel::YUV420P,
            // Pixel::RGBA,
            // Pixel::RGB24,
            Pixel::GRAY8,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        let mut frame_index = 0;

        let mut receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg_next::decoder::Video| -> Result<(), ffmpeg_next::Error> {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;

                    save_file(&rgb_frame, frame_index).unwrap();

                    //
                    let w = rgb_frame.width();
                    let h = rgb_frame.height();
                    let planes = rgb_frame.planes();
                    let mid = planes / 2;
                    // let data = rgb_frame.plane::<(u8, u8, u8, u8)>(mid);
                    let data = rgb_frame.plane::<Luma<u8>>(mid);

                    println!("#{:?} {}x{} {} {:?})", frame_index, w, h, planes, data[0]);
                    frame_index += 1;

                    if frame_index > 5 {
                        break;
                    }
                }
                Ok(())
            };

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;
                // break;
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
    }

    Ok(())
}

pub fn save_file(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
    use std::io::Write;

    let mut file = std::fs::File::create(format!("out/frame{}.pgm", index))?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(frame.data(0))?;
    Ok(())
}
