use bytes::BytesMut;

use crate::{
    codec::{compressed::CompressedCodec, error::CodecError, uncompressed::UncompressedCodec},
    packet::RawPacket,
    varint,
};

#[cfg(test)]
mod test;

// pub mod dual;
pub mod compressed;
pub mod error;
pub mod uncompressed;

// kept for backwards compatibility with old naming
// pub type MinecraftCodec = UncompressedCodec;

pub enum MinecraftCodec {
    Uncompressed(UncompressedCodec),
    Compressed(CompressedCodec),
}

impl MinecraftCodec {
    pub fn uncompressed() -> Self {
        Self::Uncompressed(Default::default())
    }
    pub fn compressed() -> Self {
        Self::Compressed(Default::default())
    }
}

fn decode_raw_packet(
    mut buf: &mut BytesMut,
    payload_length: usize,
) -> Result<RawPacket, CodecError> {
    let (packet_id_size, packet_id) = varint::read_varint(&mut buf)?;

    let data_length = payload_length
        .checked_sub(packet_id_size)
        .ok_or(CodecError::Size)?;

    let data = buf.split_to(data_length).freeze();
    Ok(RawPacket { packet_id, data })
}
