use std::collections::VecDeque;
use std::error::Error;
use std::io;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;

use bytes::Bytes;
use futures::Stream;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let b1 = Bytes::from_static("hello".to_bytes());
    // let b2 = Bytes::from_static(" world".to_bytes());
    // let x = String::new();
    // x.push_str(str::from_utf8_unchecked(&b1));

    // // Create a stream of byte chunks.
    let stream = tokio_stream::iter(vec![
        // Ok::<Bytes, io::Error>(Bytes::from(": this is a test stream\n")),
        // Ok::<Bytes, io::Error>(Bytes::from(": this is a test stream\n")),
        // Ok::<Bytes, io::Error>(Bytes::from("\n")),
        Ok::<Bytes, io::Error>(Bytes::from("data: some text\ndata: some text3\n")),
        Ok::<Bytes, io::Error>(Bytes::from("\n")),
        Ok::<Bytes, io::Error>(Bytes::from("id: 123\n")),
        Ok::<Bytes, io::Error>(Bytes::from("event: some\n")),
        Ok::<Bytes, io::Error>(Bytes::from("data: some text 2\ndata: some ")),
        Ok::<Bytes, io::Error>(Bytes::from(" text4\n")),
    ]);
    let stream = tokio_stream::StreamExt::throttle(stream, Duration::from_secs(1));

    let mut source = EventSource {
        response: Box::pin(stream),
        buffer: vec![],
        events: VecDeque::new(),
        last_id: None,
        last_type: None,
    };

    while let Some(Ok(event)) = source.next().await {
        println!("{event:?}");
    }

    // let read = StreamReader::new(stream);

    // let mut x = read.lines();
    // while let Ok(Some(line)) = x.next_line().await {
    //     println!("{line}");
    // }

    // // // Loop through the lines from the `StreamReader`.
    // // let mut line = String::new();
    // // let mut lines = Vec::new();
    // // loop {
    // //     line.clear();
    // //     let len = read.read_line(&mut line).await?;
    // //     if len == 0 {
    // //         break;
    // //     }
    // //     lines.push(line.clone());
    // // }
    println!("END");
    Ok(())
}

#[derive(Debug)]
pub struct Event {
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub data: String,
}

struct EventSource {
    response: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>>>>,
    buffer: Vec<u8>,
    events: VecDeque<Event>,
    last_id: Option<String>,
    last_type: Option<String>,
}

impl Stream for EventSource {
    type Item = Result<Event, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(event) = self.events.pop_front() {
                return Poll::Ready(Some(Ok(event)));
            }

            match self.response.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    for byte in bytes {
                        if byte == b'\n' {
                            let current_bytes = std::mem::take(&mut self.buffer);
                            let line = String::from_utf8_lossy(&current_bytes);
                            if line.is_empty() {
                                continue;
                            } else if let Some(index) = line.find(": ") {
                                let field = &line[0..index];
                                match field {
                                    "id" => self.last_id = Some(line[index + 2..].to_string()),
                                    "event" => self.last_type = Some(line[index + 2..].to_string()),
                                    "data" => {
                                        let id = self.last_id.take();
                                        let r#type = self.last_type.take();
                                        self.events.push_back(Event {
                                            id,
                                            r#type,
                                            data: line[index + 2..].to_string(),
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        } else {
                            self.buffer.push(byte);
                        }
                    }
                }
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };
        }
    }
}
