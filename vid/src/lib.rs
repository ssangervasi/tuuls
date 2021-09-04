use ffmpeg_next::{
    format::{input, Pixel},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    util::frame::video::Video,
};

use liib::position::Position;
use liib::screen::Screen;
use liib::term::{dump_screen, get_size, make_room};

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

    let size = get_size();
    let total_points = size.col * size.row;
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
    println!(
        "{}x{} -> {}x{} ({})",
        scaler.input().width,
        scaler.input().height,
        size.col,
        size.row,
        total_points,
    );

    // panic!("nah");
    make_room();

    let mut frame_index = 0;
    let mut screen = Screen::with_size(size);
    let mut last_size = 0;

    let mut receive_and_process_decoded_frames =
        |decoder: &mut ffmpeg_next::decoder::Video| -> Result<(), ffmpeg_next::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                frame_index += 1;

                let mut frame = Video::empty();
                scaler.run(&decoded, &mut frame)?;

                let plane = frame.plane::<image::Luma<u8>>(0);

                let points = plane.len();
                let excess_points = points as i32 - total_points;
                let real_cols = size.col + (excess_points / size.row);
                let calc_pos = |index: i32| -> Position {
                    let c = index % real_cols;
                    let r = index / real_cols;
                    (c, r).into()
                };

                let mut n = 0;
                let mut weirdos: [Vec<u8>; 3] = [
                    Vec::with_capacity(size.row as usize),
                    Vec::with_capacity(size.row as usize),
                    Vec::with_capacity(size.row as usize),
                ];
                for (i, point) in plane.iter().enumerate() {
                    let pos = calc_pos(i as i32);
                    let data = point.data[0];
                    let ch = data.to_char();
                    screen.write(&pos, ch);

                    let j = i as i32;
                    n = i;
                    if j % size.col == 0 {
                        weirdos[0].push(data);
                    }
                    if j % (size.col + 1) == 0 {
                        weirdos[1].push(data);
                    }
                    if (j + 1) % (size.col) == 0 {
                        weirdos[2].push(data);
                    }
                }

                if last_size != n {
                    last_size = n;
                    println!("Points: {}", n);
                    println!("Cols:   {}", size.col);
                    println!("Rows:   {}", size.row);
                    println!("Points/Cols: {}", (n as f32) / (size.col as f32));
                    println!("Points/Rows: {}", (n as f32) / (size.row as f32));
                    println!("Exesss:      {}", (n as i32) - (size.col * size.row));
                    println!("Planes:      {}", frame.planes());
                    println!("Plane W:     {}", frame.plane_width(0));
                    println!("Plane H:     {}", frame.plane_height(0));
                    println!("% col:    {:?} ({})", weirdos[0], weirdos[0].len());
                    println!("% col+1:  {:?} ({})", weirdos[1], weirdos[1].len());
                    println!("+1 % col: {:?} ({})", weirdos[2], weirdos[2].len());
                }

                dump_screen(&mut screen).unwrap();
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
