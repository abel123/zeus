use std::cell::{Ref, RefCell};
use std::cmp::min;
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender};
use tokio::task;
use tokio::task::JoinHandle;
use tracing::debug;
use tracing::{error, info};

use crate::messages::{RequestMessage, ResponseMessage};
use crate::Error;

#[derive(Debug)]
pub struct ResponseStream {
    pub(crate) receiver: UnboundedReceiver<ResponseMessage>,
    pub(crate) sender: UnboundedSender<Signal>,
    pub(crate) request_id: Option<i32>,
    order_id: Option<i32>,
    timeout: Option<Duration>,
}

impl ResponseStream {
    pub fn new(
        receiver: UnboundedReceiver<ResponseMessage>,
        sender: UnboundedSender<Signal>,
        request_id: Option<i32>,
        order_id: Option<i32>,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            receiver,
            sender,
            request_id,
            order_id,
            timeout,
        }
    }

    pub fn dispose(&self) -> impl Fn() + '_ {
        || {
            self.request_id
                .map(|x| self.sender.clone().send(Signal::Request(x)));
            self.order_id
                .map(|x| self.sender.clone().send(Signal::Order(x)));
        }
    }
}

pub(crate) struct GlobalResponseStream {}
pub trait MessageBus {
    async fn read_message(&mut self) -> Result<ResponseMessage, Error>;

    async fn write_message(&mut self, packet: &RequestMessage) -> Result<(), Error>;

    async fn send_generic_message(
        &mut self,
        sender: UnboundedSender<ResponseMessage>,
        request_id: i32,
        packet: &RequestMessage,
    ) -> Result<(), Error>;
    async fn send_durable_message(
        &mut self,
        request_id: i32,
        packet: &RequestMessage,
    ) -> Result<ResponseStream, Error>;

    async fn write(&mut self, packet: &str) -> Result<(), Error>;

    async fn process_messages(
        &mut self,
        receiver: Receiver<Item>,
        server_version: i32,
    ) -> Result<(), Error>;

    fn request_messages(&self) -> Vec<RequestMessage> {
        vec![]
    }
}

pub enum Signal {
    Request(i32),
    Order(i32),
}

#[derive(Debug)]
pub struct TcpMessageBus {
    address: String,
    reader: Rc<RefCell<OwnedReadHalf>>,
    writer: Rc<RefCell<OwnedWriteHalf>>,
    handles: Vec<JoinHandle<Result<(), Error>>>,
    requests: Rc<RefCell<SenderHash<i32, ResponseMessage>>>,
    pub(crate) signals_send: UnboundedSender<Signal>,
    signals_recv: UnboundedReceiver<Signal>,
}

pub type Item = (UnboundedSender<ResponseMessage>, i32, RequestMessage);

impl TcpMessageBus {
    pub async fn connect(address: &str) -> Result<Self, Error> {
        let stream = TcpStream::connect(address).await?;
        let (reader, writer) = stream.into_split();
        let (signals_send, signals_recv) = unbounded_channel();
        Ok(Self {
            address: String::from(address),
            reader: Rc::new(RefCell::new(reader)),
            writer: Rc::new(RefCell::new(writer)),
            handles: vec![],
            requests: Rc::new(RefCell::new(SenderHash::new())),
            signals_send,
            signals_recv,
        })
    }

    fn add_request(
        &mut self,
        request_id: i32,
        sender: UnboundedSender<ResponseMessage>,
    ) -> Result<(), Error> {
        self.requests.borrow_mut().insert(request_id, sender);
        Ok(())
    }
}
impl MessageBus for TcpMessageBus {
    async fn read_message(&mut self) -> Result<ResponseMessage, Error> {
        read_packet(&mut self.reader.borrow_mut()).await
    }

    async fn write_message(&mut self, message: &RequestMessage) -> Result<(), Error> {
        let data = message.encode();
        debug!("-> {data:?}");

        let data = data.as_bytes();

        let mut packet = Vec::with_capacity(data.len() + 4);

        packet.write_u32(data.len() as u32).await?;
        std::io::Write::write_all(&mut packet, data)?;

        self.writer.borrow_mut().write_all(&packet).await?;

        Ok(())
    }

    async fn send_generic_message(
        &mut self,
        sender: UnboundedSender<ResponseMessage>,
        request_id: i32,
        packet: &RequestMessage,
    ) -> Result<(), Error> {
        self.add_request(request_id, sender)?;
        info!("-> {:?}", packet);

        self.write_message(packet).await?;

        Ok(())
    }

    async fn send_durable_message(
        &mut self,
        request_id: i32,
        packet: &RequestMessage,
    ) -> Result<ResponseStream, Error> {
        todo!()
    }

    async fn write(&mut self, packet: &str) -> Result<(), Error> {
        self.writer
            .borrow_mut()
            .write_all(packet.as_bytes())
            .await?;
        Ok(())
    }

    async fn process_messages(
        &mut self,
        mut receiver: Receiver<Item>,
        server_version: i32,
    ) -> Result<(), Error> {
        let reader = self.reader.clone();
        let requests = self.requests.clone();
        let handle = task::spawn_local(async move {
            loop {
                match read_packet(&mut reader.borrow_mut()).await {
                    Ok(message) => {
                        match message.message_type() {
                            _ => {
                                let request_id = message.request_id().unwrap_or(-1); // pass in request id?
                                if requests.borrow().contains(&request_id) {
                                    requests.borrow_mut().send(&request_id, message).await?;
                                } else {
                                    info!("no recipient found for: {:?}", message)
                                }
                            }
                        };
                    }
                    Err(err) => {
                        error!("error reading packet: {:?}", err);
                        actix::Arbiter::current().stop();

                        return Ok(());
                    }
                };
            }
        });

        self.handles.push(handle);
        loop {
            if let Some((sender, id, msg)) = receiver.recv().await {
                self.send_generic_message(sender, id, &msg).await?;
            }
        }
    }
}

async fn read_packet(reader: &mut OwnedReadHalf) -> Result<ResponseMessage, Error> {
    let message_size = read_header(reader).await?;
    let mut data = vec![0_u8; message_size];

    reader.read_exact(&mut data).await?;

    let raw_string = String::from_utf8(data)?;
    let packet = ResponseMessage::from(&raw_string);
    if raw_string.len() > 300 {
        //debug!("<- {}", raw_string.chars().take(100).collect::<String>());
    } else {
        debug!("<- {:?}", packet);
    }

    Ok(packet)
}

async fn read_header(reader: &mut OwnedReadHalf) -> Result<usize, Error> {
    let buffer = &mut [0_u8; 4];
    reader.read_exact(buffer).await?;

    let mut reader = Cursor::new(buffer);
    let count = reader.read_u32().await?;

    Ok(count as usize)
}

#[derive(Debug)]
struct SenderHash<K, V> {
    data: HashMap<K, UnboundedSender<V>>,
}

impl<K: std::hash::Hash + Eq + std::fmt::Debug, V: std::fmt::Debug> SenderHash<K, V> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub async fn send(&self, id: &K, message: V) -> Result<(), Error> {
        let senders = &self.data;
        if let Some(sender) = senders.get(id) {
            if let Err(err) = sender.send(message) {
                error!("error sending: {id:?}, {err}")
            }
        } else {
            error!("no recipient found for: {id:?}, {message:?}")
        }
        Ok(())
    }

    pub fn copy_sender(&self, id: K) -> Option<UnboundedSender<V>> {
        self.data.get(&id).cloned()
    }

    pub fn insert(&mut self, id: K, message: UnboundedSender<V>) -> Option<UnboundedSender<V>> {
        self.data.insert(id, message)
    }

    pub fn remove(&mut self, id: &K) -> Option<UnboundedSender<V>> {
        self.data.remove(id)
    }

    pub fn contains(&self, id: &K) -> bool {
        self.data.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
