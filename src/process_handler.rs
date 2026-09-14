use crate::{TopBar, portmanteau::Portmanteau, process_handler::bar::BarHandler};
use calloop::{EventLoop, channel::Channel};
use slint::Weak;

pub mod bar;

pub fn initialise_executor(mut handler: ProcessHandler, receiver: Channel<Portmanteau>) {
    let mut event_loop = EventLoop::try_new().unwrap();
    let handle = event_loop.handle();

    handle.insert_source(receiver, |event_situation, _, data: &mut ProcessHandler| {
        match event_situation {
            calloop::channel::Event::Msg(event) => match event {
                Portmanteau::Bar(msg) => data.bar.process_message(msg),
            },
            calloop::channel::Event::Closed => {
                eprintln!("The cchannel is closed, this shouldn't have happened")
            }
        }
    });

    event_loop.run(None, &mut handler, |_| {});
}

pub struct ProcessHandler {
    bar: BarHandler,
}

impl ProcessHandler {
    pub fn new(bar_weak: Weak<TopBar>) -> Self {
        ProcessHandler {
            bar: BarHandler::new(bar_weak),
        }
    }
}
