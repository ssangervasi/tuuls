use std::thread::sleep;
use std::time::{Duration, Instant};

use ffmpeg_next::{
    format::{input, Pixel},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    util::frame::audio::Audio,
    util::frame::video::Video,
};

use liib::position::Position;
use liib::ringer::Ringer;
use liib::screen::Screen;

use liib::term::{dump_ringer, dump_screen, get_size, make_room};

pub struct Options {
    pub path: String,
    pub sound: bool,
    pub waveform: bool,
}

// pub fn term_screen(path: &str) -> Result<(), ffmpeg_next::Error> {
pub fn term_screen(options: &Options) -> Result<(), ffmpeg_next::Error> {
    ffmpeg_next::init().unwrap();

    match input(&options.path) {
        Ok(mut ictx) => handle_ictx(&mut ictx, options),
        Err(e) => {
            println!(
                "Path '{}' could not be opened as a video. {}",
                options.path, e
            );
            Err(e)
        }
    }
}

pub fn handle_ictx(
    ictx: &mut ffmpeg_next::format::context::input::Input,
    options: &Options,
) -> Result<(), ffmpeg_next::Error> {
    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg_next::Error::StreamNotFound)?;
    let video_stream_index = input.index();
    let mut decoder = input.codec().decoder().video()?;
    let target_fps = {
        let r = decoder.frame_rate().unwrap();
        (r.numerator() as f32) / (r.denominator() as f32)
    };
    let size = get_size();
    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::GRAY8,
        size.col as u32,
        size.row as u32,
        // Flags::BILINEAR,
        Flags::GAUSS,
    )?;

    let audio_input = ictx
        .streams()
        .best(Type::Audio)
        .ok_or(ffmpeg_next::Error::StreamNotFound)?;
    let audio_stream_index = audio_input.index();
    let mut audio_decoder = audio_input.codec().decoder().audio()?;

    // println!(
    //     "{}x{} -> {}x{} ({}) {}",
    //     scaler.input().width,
    //     scaler.input().height,
    //     size.col,
    //     size.row,
    //     total_points,
    //     decoder.format(),
    // );
    println!("Audio: {:?}", audio_decoder.format());

    // panic!("nope");
    make_room();

    let mut frame_count: i32 = 0;
    let mut screen = Screen::with_size(size);
    let total_points = size.col * size.row;

    let time_start = Instant::now();

    let mut process_video =
        |decoder: &mut ffmpeg_next::decoder::Video| -> Result<(), ffmpeg_next::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                frame_count += 1;

                let mut frame = Video::empty();
                scaler.run(&decoded, &mut frame)?;

                let plane = frame.plane::<image::Luma<u8>>(0);

                /*
                Apparently the plane contains spacer data points, like this:
                    00 01 02 03 04 05 __
                    07 08 09 10 11 12 __
                    14 15 16 17 18 19 __
                    21 22 23 34 25 26
                If we modulo by the desired column width, lines are skewed:
                    00 01 02 03 04 05
                    __ 07 08 09 10 11
                    12 __ 14 15 16 17
                    18 19 __ 21 22 23
                    34 25 26
                So the actual number of columns is found by dividing the excess
                accross each row. The screen will disregard the columns that are
                out of bounds.
                */

                let points = plane.len();
                let excess_points = points as i32 - total_points;
                let real_cols = size.col + (excess_points / size.row);
                let calc_pos = |index: i32| -> Position {
                    let c = index % real_cols;
                    let r = index / real_cols;
                    if options.waveform {
                        (c + 5, r).into()
                    } else {
                        (c, r).into()
                    }
                };

                for (i, point) in plane.iter().enumerate() {
                    let pos = calc_pos(i as i32);
                    let data = point.data[0];
                    let ch = data.to_char();
                    screen.write(&pos, ch);
                }

                let time_now = Instant::now();
                let time_elapsed = time_now - time_start;
                let expected_elapsed = Duration::from_secs_f32(frame_count as f32 / target_fps);
                if time_elapsed < expected_elapsed {
                    sleep(expected_elapsed - time_elapsed)
                }
                dump_screen(&mut screen).unwrap();
            }
            Ok(())
        };

    let mut ringer = Ringer::new();
    let mut wave_screen = Screen::with_size((5, size.row).into());
    let mut audio_frame_count: i32 = 0;
    let audio_range = (0.00001, 0.01);
    let audio_threshold = //
        // --
        0.0018
    ;
    let mut process_audio =
        |decoder: &mut ffmpeg_next::decoder::Audio| -> Result<(), ffmpeg_next::Error> {
            let mut decoded = Audio::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                audio_frame_count += 1;

                let point: f32 = decoded.plane::<f32>(0).iter().sum::<f32>()
                    / decoded.plane::<f32>(0).len() as f32;
                if audio_threshold < point {
                    ringer.ring();
                }

                if options.sound {
                    dump_ringer(&mut ringer).unwrap();
                }

                let level: i32 = (((point - audio_range.0) / (audio_range.1 - audio_range.0))
                    * (wave_screen.rows as f32)) as i32;

                if audio_frame_count % 5 == 0 {
                    wave_screen.clear();
                }

                wave_screen.write(&(audio_frame_count % wave_screen.cols, level).into(), 'O');
                if options.waveform {
                    dump_screen(&mut wave_screen).unwrap();
                }

                decoder.flush();
            }
            Ok(())
        };

    let mut packet_count = 0;
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            packet_count += 1;

            decoder.send_packet(&packet).unwrap();
            process_video(&mut decoder)?;
        } else if stream.index() == audio_stream_index {
            audio_decoder.send_packet(&packet).unwrap();
            process_audio(&mut audio_decoder)?;
        }
    }
    decoder.send_eof()?;
    process_video(&mut decoder)?;

    println!("Packets: {} | Frames: {}", packet_count, frame_count);
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
