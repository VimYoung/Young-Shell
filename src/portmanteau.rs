use crate::process_handler::bar::BarMessage;
use slint::SharedString;

pub enum Portmanteau {
    Bar(BarMessage),
    Async(AsyncMessage),
}

pub enum AsyncMessage {
    FetchNetworkInfo,
    Connect {
        ssid: SharedString,
        pass: SharedString,
    },
    Disconnect,
    WifiToggle(bool),
    ForgetNetwork(SharedString),
    RescanNetwork,
}
