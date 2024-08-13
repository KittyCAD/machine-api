mod no_auth;
mod rtsps;

use bytes::Bytes;
use nom::bytes::streaming::take_until;
use openh264::{decoder::Decoder, formats::YUVSource};
use rtp::packetizer::Depacketizer;
use rtsps::Rtsps;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use webrtc_util::marshal::{MarshalSize, Unmarshal};

/// Parse a packet beginning with a 4 byte header: `0x24, 0x00, len low, len high` and return the payload.
///
/// Not sure what the `0x2400` represents as I can't find this structure in any of the RTP/H.264
/// docs, but it delimits every RTP frame.
fn parse_packet<'buf>(i: &'buf [u8]) -> nom::IResult<&'buf [u8], Bytes> {
    use nom::{
        bytes::streaming::{tag, take},
        number::streaming::be_u16,
    };

    // Discard any preceding bytes before the start marker
    let (i, _before) = take_until(&[0x24u8, 0x00][..])(i)?;

    // Start marker 0x2400 (big endian)
    let (i, _marker) = tag([0x24u8, 0x00])(i)?;

    // Data length
    let (i, len) = be_u16(i)?;

    let len = usize::from(len);

    // Main data payload
    let (i, chunk) = take(len)(i)?;

    Ok((i, Bytes::from(chunk.to_vec())))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut stream = Rtsps::new(
        // User is hard coded to `bblp`. Password is printer access code.
        "rtsps://bblp:192190e7@192.168.0.96:322/streaming/live/1",
    )
    .await?;

    let mut openh264 = Decoder::new()?;

    stream.open_stream("/streaming/live/1").await?;

    for _ in 0..32 {
        stream.read_more().await?;
    }

    // ---

    let sdl_context = sdl2::init().expect("Error sdl2 init");
    let video_subsystem = sdl_context.video().expect("Error sld2 video subsystem");

    let window = video_subsystem
        .window("Bambu stream", 1168, 720)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_static(PixelFormatEnum::IYUV, 1168, 720)?;
    let mut event_pump = sdl_context.event_pump().expect("Error sld2 event");

    // ---

    let mut decoder = rtp::codecs::h264::H264Packet::default();

    let mut b = Vec::new();

    while let Ok(next_chunk) = stream.read_more().await {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    log::info!("Exiting...");

                    break;
                }
                _ => {}
            }
        }

        b.extend_from_slice(next_chunk);

        let (rest, mut chunk) = match parse_packet(&b) {
            Ok(res) => res,
            Err(nom::Err::Incomplete(needed)) => {
                log::debug!("Need {:?} more bytes", needed);

                continue;
            }
            Err(nom::Err::Error(e)) => {
                log::warn!("Nom error {:?}", e);

                continue;
            }
            Err(nom::Err::Failure(e)) => {
                log::error!("Nom failure {:?}", e);

                return Err(anyhow::anyhow!("Packet parse failure: {:?}", e.code));
            }
        };

        log::debug!("Chunk len {}, rest {}", chunk.len(), rest.len());

        // Strip first successfully parsed chunk from the beginning of the buffer
        b = rest.to_vec();

        if let Ok(p) = rtp::packet::Packet::unmarshal(&mut chunk) {
            log::debug!(
                "Decoded a packet SN {}, TS {}, PT {} payload {} first 32 bytes: {:02x?}",
                p.header.sequence_number,
                p.header.timestamp,
                p.header.payload_type,
                p.marshal_size(),
                &p.payload[0..p.payload.len().min(32)]
            );

            match decoder.depacketize(&p.payload) {
                Ok(bytes) if !bytes.is_empty() => {
                    if let Ok(Some(f)) = openh264.decode(&bytes) {
                        log::info!("Got a frame: {:?}", f.dimensions());

                        let (y_size, u_size, v_size) = f.strides();

                        texture.update_yuv(None, f.y(), y_size, f.u(), u_size, f.v(), v_size)?;

                        canvas.clear();

                        canvas
                            .copy(&texture, None, None)
                            .expect("Error copying texture");

                        canvas.present();
                    }
                }
                Ok(_) => (),
                Err(e) => log::error!("Depacketize error {:?}", e),
            }
        } else {
            log::error!("Failed to unmarshal chunk {}", chunk.len());
        }
    }

    Ok(())
}
