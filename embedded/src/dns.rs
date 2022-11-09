use cyw43::NetDevice;
use defmt::{info, warn};
use dns_protocol::{Question, ResourceType, Message, Flags, ResourceRecord};
use embassy_net::{Stack, udp::UdpSocket, Ipv4Address, PacketMetadata, Ipv6Address};
use embedded_io::asynch::Write;
use embassy_net::IpAddress;

pub async fn dns_request<'a>(stack: &Stack<NetDevice<'a>>, addr: &str) -> Option<IpAddress> {


    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(53).unwrap();


    let mut questions = [Question::new(addr, ResourceType::A, 1)];
    let message = Message::new(
        0xFEE7,
        Flags::standard_query(),
        &mut questions,
        &mut [],
        &mut [],
        &mut [],
    );

    let mut buffer = [0; 500];

    let len = message.write(&mut buffer).unwrap();


    socket.send_to(&buffer[..len], (Ipv4Address::new(1, 1, 1, 1), 53)).await.unwrap();

    let (len, _) = socket.recv_from(&mut buffer).await.unwrap();


    // Parse the response.
    let mut answers = [ResourceRecord::default(); 16];
    let mut authority = [ResourceRecord::default(); 16];
    let mut additional = [ResourceRecord::default(); 16];
    let message = Message::read(
            &buffer[..len],
            &mut questions,
            &mut answers,
            &mut authority,
            &mut additional,
    ).unwrap();


    info!("Got answers: {}", message.answers().len());

    answers.iter().find_map(|addr| {
        let data = addr.data();
        info!("Trying to parse ip: {:?}", data);
        match data.len() {
            4 => {
                Some(Ipv4Address::from_bytes(&data[0..4]).into())
            }
            16 => {
                Some(Ipv6Address::from_bytes(data).into())
            }
            _ => None
        }
    })
}
