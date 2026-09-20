use crate::{
    TopBar, WifiLineData,
    portmanteau::{AsyncMessage, Portmanteau},
    process_handler::bar::BarHandler,
};
use calloop::{EventLoop, channel::Channel};
use nmrs::{NetworkManager, WifiSecurity};
use slint::{Model, SharedString, Weak};

pub mod bar;

pub fn initialise_executor(
    bar_x: Weak<TopBar>,
    receiver: Channel<Portmanteau>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (async_executor, async_scheduler) =
        calloop::futures::executor::<Result<(), Box<dyn std::error::Error>>>()
            .expect("Failed to create async executor");
    let mut handler = ProcessHandler::new(bar_x);
    let mut event_loop = EventLoop::try_new().unwrap();
    let handle = event_loop.handle();
    calloop::futures::executor::<AsyncMessage>().expect("Unable to create an async scheduler");

    handle.insert_source(
        receiver,
        move |event_situation, _, data: &mut ProcessHandler| match event_situation {
            calloop::channel::Event::Msg(event) => match event {
                Portmanteau::Bar(msg) => data.bar.process_message(msg),
                Portmanteau::Async(msg) => match msg {
                    AsyncMessage::FetchNetworkInfo => async_scheduler
                        .schedule(fetch_network_info(data.bar.get_bar_instance_weak()))
                        .expect("Message scheduling failed"),
                },
            },
            calloop::channel::Event::Closed => {
                eprintln!("The cchannel is closed, this shouldn't have happened")
            }
        },
    )?;

    handle.insert_source(async_executor, |result, _, _| {
        if let Err(err) = result {
            eprintln!("Received error when executing future: {}", err);
        }
    })?;

    event_loop.run(None, &mut handler, |_| {})?;
    Ok(())
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

async fn fetch_network_info(bar: Weak<TopBar>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello world");

    // FIXME: This shouldnt be generated on each connection.
    let nm = NetworkManager::new().await?;
    let networks = nm.list_networks(None).await?;

    let wifi_lines: Vec<WifiLineData> = networks
        .iter()
        .map(|network| WifiLineData {
            connection_status: crate::ConnectionStatus::Known,
            name: SharedString::from(network.ssid.clone()),
            pass_error: false,
            strength: (network.strength.unwrap_or_default() / 100) as f32,
        })
        .collect();
    for net in &networks {
        println!("{} - Signal: {}%", net.ssid, net.strength.unwrap_or(0));
    }
    //
    // // Connect to a network on the first Wi-Fi device
    // nm.connect(
    //     "MyNetwork",
    //     None,
    //     WifiSecurity::WpaPsk {
    //         psk: "password123".into(),
    //     },
    // )
    // .await?;
    //
    // // Check current connection
    // if let Some(ssid) = nm.current_ssid().await {
    //     println!("Connected to: {}", ssid);
    // }

    bar.upgrade_in_event_loop(move |ui| {
        let model = ui.get_wifi_lines();
        let model = model
            .as_any()
            .downcast_ref::<slint::VecModel<WifiLineData>>()
            .expect("couldn't downref");
        model.set_vec(wifi_lines);
    })?;
    Ok(())
}
