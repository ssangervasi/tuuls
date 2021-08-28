use ffmpeg_next::{
    format::{input, Pixel},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    util::frame::video::Video,
};

fn main() -> Result<(), ffmpeg_next::Error> {
    term_screen()
}

pub fn term_screen() -> Result<(), ffmpeg_next::Error> {
    ffmpeg_next::init().unwrap();

    let path = "/home/sebastian/Videos/Webcam/wave.webm".to_string();
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
            decoder.width() / 4,
            decoder.height() / 8,
            Flags::BILINEAR,
        )?;
        println!(
            "{}x{} -> {}x{}",
            scaler.input().width,
            scaler.input().height,
            scaler.output().width,
            scaler.output().height
        );

        let mut frame_index = 0;

        let mut receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg_next::decoder::Video| -> Result<(), ffmpeg_next::Error> {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut frame = Video::empty();
                    scaler.run(&decoded, &mut frame)?;

                    save_file(&frame, frame_index).unwrap();

                    //
                    let w = frame.width();
                    let _h = frame.height();
                    // let data = frame.plane::<(u8, u8, u8, u8)>(mid);
                    // println!("#{:?} {}x{} {}", frame_index, w, h, planes);

                    let mut screen = liib::Screen::default();

                    let plane = frame.plane::<image::Luma<u8>>(0);
                    for (i, point) in plane.iter().enumerate() {
                        let coord = (i as i32 % w as i32, i as i32 / w as i32);
                        let ch = point.data[0].to_char();
                        screen.write(coord, ch);
                    }

                    liib::dump_screen(screen).unwrap();

                    // if frame_index > 5 {
                    //     break;
                    // }
                    frame_index += 1;
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

pub fn one_frame() -> Result<(), ffmpeg_next::Error> {
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
            decoder.width() / 4,
            decoder.height() / 8,
            Flags::BILINEAR,
        )?;
        println!(
            "{}x{} -> {}x{}",
            scaler.input().width,
            scaler.input().height,
            scaler.output().width,
            scaler.output().height
        );

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
                    // let data = rgb_frame.plane::<(u8, u8, u8, u8)>(mid);
                    println!("#{:?} {}x{} {}", frame_index, w, h, planes);

                    let plane = rgb_frame.plane::<image::Luma<u8>>(0);
                    for (i, point) in plane.iter().enumerate() {
                        print!("{}", point.data[0].to_char());
                        if i % (w as usize) == 0 {
                            println!()
                        }
                    }

                    if frame_index > 5 {
                        break;
                    }
                    frame_index += 1;
                }
                Ok(())
            };

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;
                break;
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
    }

    Ok(())
}

trait GrayText {
    fn to_char(self) -> char;
}

impl GrayText for u8 {
    fn to_char(self) -> char {
        match self / (255 / 8) {
            0 => ' ',
            1 => '·',
            2 => '-',
            4 => 'x',
            5 => '━',
            6 => '✖',
            7 => '8',
            _ => ' ',
        }
    }
}

#[test]
fn test_gray_text() {
    assert_eq!(0.to_char(), ' ');
    assert_eq!(1.to_char(), ' ');

    assert_eq!(63.to_char(), '-');

    assert_eq!(255.to_char(), '8');
}

pub fn save_file(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
    use std::io::Write;

    let mut file = std::fs::File::create(format!("out/frame{}.pgm", index))?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(frame.data(0))?;
    Ok(())
}
