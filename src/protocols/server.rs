use std::io::Cursor;
use std::sync::Arc;
use log::error;
use crate::prudp::packet::PRUDPPacket;
use crate::prudp::socket::{ConnectionData, SocketData};
use crate::rmc::message::RMCMessage;
use crate::rmc::response::{RMCResponse, RMCResponseResult, send_response};
use crate::rmc::response::ErrorCode::Core_NotImplemented;

type ContainedProtocolList = Box<[Box<dyn Fn(&RMCMessage) -> Option<RMCResponse> + Send + Sync>]>;

pub struct RMCProtocolServer(ContainedProtocolList);

impl RMCProtocolServer{
    pub fn new(protocols: ContainedProtocolList) -> Arc<Self>{
        Arc::new(Self(protocols))
    }

    pub async fn process_message(&self, packet: PRUDPPacket, socket: &SocketData, connection: &mut ConnectionData){
        let Ok(rmc) = RMCMessage::new(&mut Cursor::new(&packet.payload)) else {
            error!("error reading rmc message");
            return;
        };

        println!("recieved rmc message: {{ protocol: {}, method: {}}}", rmc.protocol_id, rmc.method_id);

        for proto in &self.0 {
            if let Some(response) = proto(&rmc) {
                send_response(&packet, &socket, connection, response).await;
                return;
            }
        }

        send_response(&packet, &socket, connection, RMCResponse{
            protocol_id: rmc.protocol_id as u8,
            response_result: RMCResponseResult::Error {
                call_id: rmc.call_id,
                error_code: Core_NotImplemented
            }
        }).await;
    }
}