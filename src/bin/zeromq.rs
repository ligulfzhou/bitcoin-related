use std::time::Duration;
use zeromq::{Socket, SocketRecv, SocketSend, SubSocket, ZmqMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut sub = SubSocket::new();
    sub.connect("tcp://127.0.0.1:28332").await?;
    sub.subscribe("hashblock").await?;
    sub.subscribe("hashtx").await?;

    loop {
        println!("waiting...");
        let msg = sub.recv().await?;
        let topic = msg.get(0).unwrap();
        let payload = msg.get(1).unwrap();
        println!("recv {:?}", msg);
        println!(
            "topic: {:?}, payload: {:?}",
            String::from_utf8(topic.to_vec()).unwrap(),
            // payload.to_lower_hex_string()
            hex::encode(payload.as_ref())
        );
    }
}

async fn test() -> anyhow::Result<()> {
    let handler = tokio::spawn(async {
        // let mut rng = rand::thread_rng();
        let stocks: Vec<&str> = vec!["AAA", "ABB", "BBB"];
        println!("Starting server");
        let mut socket = zeromq::PubSocket::new();
        socket.bind("tcp://127.0.0.1:5550").await.unwrap();

        for stock in &stocks {
            let mut m: ZmqMessage = ZmqMessage::from(*stock);
            dbg!(m.clone());
            socket.send(m).await.unwrap();

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        // tokio::time::sleep(Duration::from_secs(5)).await;
        // socket.close()
    });

    tokio::time::sleep(Duration::new(1, 0)).await;
    dbg!("start to recv data...");
    let mut socket = zeromq::SubSocket::new();
    socket
        .connect("tcp://127.0.0.1:5550")
        .await
        .expect("Failed to connect");

    socket.subscribe("").await.unwrap();
    // ...or...
    // let stocks: Vec<&str> = vec!["AAA", "ABB", "BBB"];
    // for stock in stocks {
    //     socket.subscribe(stock).await.unwrap();
    // }

    for index in 0..2 {
        dbg!(index);
        let msg = socket.recv().await?;
        dbg!(msg);
        // let stock: String = String::from_utf8(recv.get(0).unwrap().to_vec())?;
        // dbg!("{}", stock);
    }

    handler.await.unwrap();
    Ok(())
}
