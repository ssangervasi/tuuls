use ffmpeg_next::{
    format::{input, Pixel},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    util::frame::video::Video,
};

pub fn term_screen(path: &str) -> Result<(), ffmpeg_next::Error> {
    ffmpeg_next::init().unwrap();

    match input(&path) {
        Ok(mut ictx) => handle_ictx(&mut ictx),
        Err(e) => {
            println!("Path '{}' could not be opened as a video. {}", path, e);
            Err(e)
        }
    }
}

pub fn handle_ictx(
    ictx: &mut ffmpeg_next::format::context::input::Input,
) -> Result<(), ffmpeg_next::Error> {
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
        decoder.width() / 6,
        decoder.height() / 13,
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
                frame_index += 1;

                // if frame_index < (10 * 60 * 4) {
                //     println!("{}", frame_index);
                //     continue;
                // } else if frame_index > (30 * 60 * 1) {
                //     break;
                // }
                // println!("yay");

                let mut frame = Video::empty();
                scaler.run(&decoded, &mut frame)?;

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
                    screen.write(&coord.into(), ch);
                }

                liib::dump_screen(screen).unwrap();
            }
            Ok(())
        };

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            receive_and_process_decoded_frames(&mut decoder)?;
        }
    }
    decoder.send_eof()?;
    receive_and_process_decoded_frames(&mut decoder)?;

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
