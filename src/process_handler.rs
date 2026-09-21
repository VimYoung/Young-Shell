use crate::{
    ConnectionStatus, TopBar, WifiLineData,
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
                        .expect("Executor doesn't exist"),
                    AsyncMessage::Connect { ssid, pass } => async_scheduler
                        .schedule(connect(ssid, pass))
                        .expect("Executor doesn't exist"),
                    AsyncMessage::Disconnect => async_scheduler
                        .schedule(disconnect())
                        .expect("Executor doesn't exist"),
                    AsyncMessage::ForgetNetwork(ssid) => async_scheduler
                        .schedule(forget_network(ssid))
                        .expect("Executor doesn't exist"),
                    AsyncMessage::RescanNetwork => async_scheduler
                        .schedule(rescan_networks())
                        .expect("Executor doesn't exist"),
                    AsyncMessage::WifiToggle(wifi_on) => async_scheduler
                        .schedule(wifi_toggle(wifi_on))
                        .expect("Executor doesn't exist"),
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
    // FIXME: This shouldnt be generated on each connection.
    let nm = NetworkManager::new().await?;
    let mut networks = nm.list_networks(None).await?;
    networks.sort_by_key(|key| key.ssid.clone());
    let connected_network = networks
        .iter()
        .position(|network| network.is_active)
        .map(|x| x as i32)
        .unwrap_or(-1);

    // FIXME:: No way to manage and display if ethernet is in use.
    let wifi_lines: Vec<WifiLineData> = networks
        .iter()
        .map(|network| {
            let connection_status = if network.is_active {
                ConnectionStatus::Connected
            } else if network.known {
                ConnectionStatus::Known
            } else {
                ConnectionStatus::Unknown
            };
            WifiLineData {
                connection_status,
                name: SharedString::from(network.ssid.clone()),
                pass_error: false,
                strength: network.strength.unwrap_or_default() as f32,
            }
        })
        .collect();
    // for net in &networks {
    //     println!("{} - Signal: {}%", net.ssid, net.strength.unwrap_or(0));
    // }
    //
    let bar_c = bar.clone();
    // FIXME: This will also change the index if the number of connections in
    // vicinity changes.
    slint::invoke_from_event_loop(move || {
        bar_c.unwrap().set_connected_network(connected_network);
    })?;

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

async fn connect(ssid: SharedString, pass: SharedString) -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    nm.connect(
        ssid.as_str(),
        None,
        WifiSecurity::WpaPsk {
            psk: pass.as_str().into(),
        },
    )
    .await?;
    Ok(())
}

async fn disconnect() -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    nm.disconnect(None).await?;
    Ok(())
}

async fn wifi_toggle(wifi_on: bool) -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    nm.set_wireless_enabled(wifi_on).await?;
    Ok(())
}

async fn forget_network(ssid: SharedString) -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    nm.forget(ssid.as_str()).await?;
    Ok(())
}

async fn rescan_networks() -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    nm.scan_networks(None).await?;
    Ok(())
}
