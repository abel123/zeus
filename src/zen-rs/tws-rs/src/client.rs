use std::io::Write;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use byteorder::{BigEndian, WriteBytesExt};
use derivative::Derivative;
use time::macros::format_description;
use time::OffsetDateTime;
use time_tz::{timezones, OffsetResult, PrimitiveDateTimeExt, Tz};
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedSender};
use tracing::{error, info};

use crate::client::transport::message_bus::{ResponseStream, Signal};
use crate::client::transport::{Item, MessageBus, TcpMessageBus};
use crate::messages::{IncomingMessages, OutgoingMessages, RequestMessage, ResponseMessage};
use crate::{server_versions, Error};

pub mod market_data;
mod transport;

const MIN_SERVER_VERSION: i32 = 100;
const MAX_SERVER_VERSION: i32 = server_versions::HISTORICAL_SCHEDULE;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Client {
    /// IB server version
    pub(crate) server_version: i32,
    /// IB Server time
    //    pub server_time: OffsetDateTime,
    pub(crate) connection_time: Option<OffsetDateTime>,
    #[derivative(Debug = "ignore")]
    pub(crate) time_zone: Option<&'static Tz>,

    address: String,
    managed_accounts: String,
    client_id: i32, // ID of client.
    #[derivative(Debug = "ignore")]
    pub(crate) message_bus: Option<TcpMessageBus>,
    order_id: i32, // Next available order_id. Starts with value returned on connection.
    #[derivative(Debug = "ignore")]
    receiver: Option<Receiver<Item>>,
}

pub struct ClientRef {
    sender: Sender<Item>,
    signals_send: Option<UnboundedSender<Signal>>,
    time_zone: Option<&'static Tz>,
    pub(crate) server_version: i32,
    next_request_id: AtomicI32, // Next available request_id.
}

impl ClientRef {
    pub async fn send(
        &self,
        request_id: i32,
        msg: RequestMessage,
    ) -> Result<ResponseStream, Error> {
        let (sender, receiver) = unbounded_channel();
        self.sender
            .send((sender, request_id, msg))
            .await
            .map_err(|e| Error::NotImplemented)?;
        let signals_send = self.signals_send.clone();
        Ok(ResponseStream::new(
            receiver,
            signals_send.unwrap(),
            Some(request_id),
            None,
            Some(Duration::from_secs(10)),
        ))
    }

    pub(crate) fn check_server_version(&self, version: i32, message: &str) -> Result<(), Error> {
        if version <= self.server_version {
            Ok(())
        } else {
            Err(Error::ServerVersion(
                version,
                self.server_version,
                message.into(),
            ))
        }
    }

    pub(crate) fn server_version(&self) -> i32 {
        self.server_version
    }

    pub(crate) fn next_request_id(&self) -> i32 {
        self.next_request_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl Client {
    pub fn new(address: &str, client_id: i32) -> Self {
        Self {
            server_version: 0,
            connection_time: None,
            time_zone: None,
            address: String::from(address),
            managed_accounts: "".to_string(),
            client_id,
            message_bus: None,
            order_id: -1,
            receiver: None,
        }
    }

    pub async fn connect(&mut self) -> Result<ClientRef, Error> {
        let bus = TcpMessageBus::connect(self.address.as_str()).await?;
        let signals_send = bus.signals_send.clone();
        self.message_bus = Some(bus);
        self.handshake().await?;
        self.start_api().await?;
        self.receive_account_info().await?;

        let (sender, receiver) = channel(2048);

        self.receiver = Some(receiver);
        info!("{:?}", self);
        Ok(ClientRef {
            sender,
            signals_send: Some(signals_send),
            time_zone: self.time_zone.clone(),
            server_version: self.server_version,
            next_request_id: AtomicI32::from(9000),
        })
    }

    pub async fn blocking_process<F>(&mut self, onerror: F) -> Result<(), Error>
    where
        F: Fn(ResponseMessage) + 'static,
    {
        self.message_bus
            .as_mut()
            .unwrap()
            .process_messages(self.receiver.take().unwrap(), self.server_version, onerror)
            .await?;
        Ok(())
    }
    async fn handshake(&mut self) -> Result<(), Error> {
        let prefix = "API\0";
        let version = format!("v{MIN_SERVER_VERSION}..{MAX_SERVER_VERSION}");

        let packet = prefix.to_owned() + &encode_packet(&version);
        self.message_bus.as_mut().unwrap().write(&packet).await?;

        let ack = self.message_bus.as_mut().unwrap().read_message().await;

        return match ack {
            Ok(mut response_message) => {
                self.server_version = response_message.next_int()?;

                let time = response_message.next_string()?;
                (self.connection_time, self.time_zone) = parse_connection_time(time.as_str());
                Ok(())
            }
            Err(Error::Io(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                Err(Error::Simple(format!(
                    "The server may be rejecting connections from this host: {err}"
                )))
            }
            Err(err) => Err(err),
        };
    }

    // asks server to start processing messages
    async fn start_api(&mut self) -> Result<(), Error> {
        const VERSION: i32 = 2;

        let prelude = &mut RequestMessage::default();

        prelude.push_field(&OutgoingMessages::StartApi);
        prelude.push_field(&VERSION);
        prelude.push_field(&self.client_id);

        if self.server_version > server_versions::OPTIONAL_CAPABILITIES {
            prelude.push_field(&"");
        }

        self.message_bus
            .as_mut()
            .unwrap()
            .write_message(prelude)
            .await?;

        Ok(())
    }

    // Fetches next order id and managed accounts.
    async fn receive_account_info(&mut self) -> Result<(), Error> {
        let mut saw_next_order_id: bool = false;
        let mut saw_managed_accounts: bool = false;

        let mut attempts = 0;
        const MAX_ATTEMPTS: i32 = 100;
        loop {
            let mut message = self.message_bus.as_mut().unwrap().read_message().await?;

            match message.message_type() {
                IncomingMessages::NextValidId => {
                    saw_next_order_id = true;

                    message.skip(); // message type
                    message.skip(); // message version

                    self.order_id = message.next_int()?;
                }
                IncomingMessages::ManagedAccounts => {
                    saw_managed_accounts = true;

                    message.skip(); // message type
                    message.skip(); // message version

                    self.managed_accounts = message.next_string()?;
                }
                IncomingMessages::Error => {
                    error!("message: {message:?}")
                }
                _ => info!("message: {message:?}"),
            }

            attempts += 1;
            if (saw_next_order_id && saw_managed_accounts) || attempts > MAX_ATTEMPTS {
                break;
            }
        }

        Ok(())
    }
}

// Parses following format: 20230405 22:20:39 PST
fn parse_connection_time(connection_time: &str) -> (Option<OffsetDateTime>, Option<&'static Tz>) {
    let parts: Vec<&str> = connection_time.split(' ').collect();

    let mut zones = timezones::find_by_name(parts[2]);
    if zones.is_empty() {
        if parts[2] == "中国标准时间" {
            zones = timezones::find_by_name("China Standard Time")
        } else {
            error!("time zone not found for {}", parts[2]);
            return (None, None);
        }
    }

    let timezone = zones[0];

    let format = format_description!("[year][month][day] [hour]:[minute]:[second]");
    let date_str = format!("{} {}", parts[0], parts[1]);
    let date = time::PrimitiveDateTime::parse(date_str.as_str(), format);
    match date {
        Ok(connected_at) => match connected_at.assume_timezone(timezone) {
            OffsetResult::Some(date) => (Some(date), Some(timezone)),
            _ => {
                error!("error setting timezone");
                (None, Some(timezone))
            }
        },
        Err(err) => {
            error!("could not parse connection time from {date_str}: {err}");
            return (None, Some(timezone));
        }
    }
}

fn encode_packet(message: &str) -> String {
    let data = message.as_bytes();

    let mut packet: Vec<u8> = Vec::with_capacity(data.len() + 4);

    packet.write_u32::<BigEndian>(data.len() as u32).unwrap();
    packet.write_all(data).unwrap();

    std::str::from_utf8(&packet).unwrap().into()
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::task::{LocalSet};
    use tokio::time::sleep;
    use tokio::time::Instant;
    use tracing::{info, warn};
    use tracing_test::traced_test;

    use crate::client::market_data::historical::{
        historical_data, BarSize, TWSDuration, WhatToShow,
    };
    use crate::client::market_data::realtime::{req_mkt_data, ReqMktDataParam};
    use crate::client::Client;
    use crate::contracts::{contract_details, sec_def_opt, Contract, ReqSecDefOptParams};
    use crate::Error;

    #[traced_test]
    #[tokio::test]
    async fn it_works() -> Result<(), Error> {
        let mut client = Client::new("127.0.0.1:14001", 4322);
        let client_ref = client.connect().await?;

        let local = LocalSet::new();
        local.spawn_local(async move {
            sleep(Duration::from_secs(2)).await;
            let now = Instant::now();
            let binding = Contract::stock("TSLA");
            let bars = historical_data(
                &client_ref,
                &binding,
                None,
                TWSDuration::days(3),
                BarSize::Min3,
                Some(WhatToShow::Trades),
                true,
                true,
            )
            .await
            .unwrap();
            info!("cost {:?}, bars: {:?}", now.elapsed(), bars.0);

            let contracts = contract_details(&client_ref, &binding).await.unwrap();
            info!("contracts {:?}", contracts);

            for _ in 0..5 {
                let params = sec_def_opt(
                    &client_ref,
                    &ReqSecDefOptParams {
                        underlying_symbol: contracts[0].contract.symbol.clone(),
                        fut_fop_exchange: "".to_string(),
                        underlying_sec_type: contracts[0].contract.security_type.to_string(),
                        underlying_con_id: contracts[0].contract.contract_id,
                    },
                )
                .await;
                info!("params {:?}", params.unwrap()[0]);
            }

            let ticker = req_mkt_data(
                &client_ref,
                &ReqMktDataParam {
                    contract: binding.clone(),
                    generic_tick_list: Default::default(),
                    snapshot: false,
                    regulatory_snapshot: false,
                    mkt_data_options: vec![],
                },
            )
            .await;
            warn!("here");
        });
        let res = local.spawn_local(async move {
            client.blocking_process(|x| {}).await?;
            sleep(Duration::from_secs(5)).await;
            Result::<(), Error>::Ok(())
        });
        local.await;
        info!("{:?}", res);
        Ok(())
    }
}
