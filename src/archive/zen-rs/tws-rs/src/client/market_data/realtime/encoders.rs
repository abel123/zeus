use crate::client::market_data::realtime::ReqMktDataParam;
use crate::contracts::SecurityType;
use crate::messages::{OutgoingMessages, RequestMessage};

pub(crate) fn encode_req_mkt_data(request_id: i32, req: &ReqMktDataParam) -> RequestMessage {
    let mut msg = RequestMessage::default();

    let version = 11;

    let message_id: i32 = OutgoingMessages::RequestMarketData as i32;

    // send req mkt data msg
    msg.push_field(&message_id);
    msg.push_field(&version);
    msg.push_field(&request_id);

    // send contract fields
    msg.push_field(&req.contract.contract_id);
    msg.push_field(&req.contract.symbol);

    msg.push_field(&req.contract.security_type);
    msg.push_field(&req.contract.last_trade_date_or_contract_month);
    msg.push_field(&req.contract.strike);
    msg.push_field(&req.contract.right);
    msg.push_field(&req.contract.multiplier); // srv v15 and above
    msg.push_field(&req.contract.exchange);
    msg.push_field(&req.contract.primary_exchange); // srv v14 and above
    msg.push_field(&req.contract.currency);
    msg.push_field(&req.contract.local_symbol); //  srv v2 and above

    msg.push_field(&req.contract.trading_class);
    // Send combo legs for BAG requests(srv v8 and above)
    if req.contract.security_type == SecurityType::Spread {
        let combo_legs_count = req.contract.combo_legs.len();
        msg.push_field(&combo_legs_count);
        for combo_leg in &req.contract.combo_legs {
            msg.push_field(&combo_leg.contract_id);
            msg.push_field(&combo_leg.ratio);
            msg.push_field(&combo_leg.action);
            msg.push_field(&combo_leg.exchange);
        }
    }

    if req.contract.delta_neutral_contract.is_some() {
        msg.push_field(&true);
        msg.push_field(
            &req.contract
                .delta_neutral_contract
                .as_ref()
                .unwrap()
                .contract_id,
        );
        msg.push_field(&req.contract.delta_neutral_contract.as_ref().unwrap().delta);
        msg.push_field(&req.contract.delta_neutral_contract.as_ref().unwrap().price);
    } else {
        msg.push_field(&false);
    }

    msg.push_field(
        &req.generic_tick_list
            .iter()
            .map(|t| format!("{}", *t as i32))
            .collect::<Vec<_>>()
            .join(","),
    );
    msg.push_field(&req.snapshot); // srv v35 and above

    msg.push_field(&req.regulatory_snapshot);

    msg.push_field(&req.mkt_data_options);

    msg
}
