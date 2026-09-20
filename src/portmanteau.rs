use crate::process_handler::bar::BarMessage;

pub enum Portmanteau {
    Bar(BarMessage),
    Async(AsyncMessage),
}

pub enum AsyncMessage {
    FetchNetworkInfo,
}
