use cyw43::NetDevice;
use defmt::{info, warn};
use dns_protocol::{Question, ResourceType, Message, Flags, ResourceRecord};
use embassy_net::{Stack, tcp::TcpSocket, Ipv4Address};
use embedded_io::asynch::Write;

pub async fn dns_request<'a>(stack: &Stack<NetDevice<'a>>) -> Option<[u8; 4]> {


    let mut rx_buffer = [0; 512];
    let mut tx_buffer = [0; 512];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

    info!("Connecting to 1.1.1.1....");
    if let Err(e) = socket
        .connect((Ipv4Address::new(1, 1, 1, 1), 53))
        .await
    {
        warn!("error: {:?}", e);
        return None;
    }

    info!("Received connection from {:?}", socket.remote_endpoint());

    let mut questions = [Question::new("hellopaint.io.", ResourceType::A, 1)];
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



    socket.write_all(&buffer[..len]).await.unwrap();


    let len = socket.read(&mut buffer).await.unwrap();


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

    answers.first().map(|addr| {
        let mut data = [0; 4];
        data.copy_from_slice(addr.data());
        data
    })
}
