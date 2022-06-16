use std::fmt::Write;
use std::process::Stdio;
use tokio::{fs::File, io::AsyncWriteExt, process::Command};

use crate::{interface::InterfaceConf, state::State};

const PATH: &str = "./interfaces.toml";

pub fn read_config() -> State {
    let config = std::fs::read_to_string(PATH).unwrap();
    let state: State = toml::from_str(&config).unwrap();
    return state;
}

pub async fn write_config(state: &State) {
    let config = toml::to_string(&state).unwrap();
    tokio::fs::write(PATH, config.as_bytes()).await.unwrap();
}

pub async fn get_keys() -> (String, String) {
    let output = Command::new("wg")
        .arg("genkey")
        .output()
        .await
        .expect("Failed to execute wg genkey");

    let private_key = String::from_utf8(output.stdout).unwrap().trim().to_string();

    let mut command = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute wg pubkey");

    command
        .stdin
        .as_mut()
        .expect("Failed to get stdin for wg pubkey")
        .write_all(private_key.as_bytes())
        .await
        .expect("Failed to write private key to wg pubkey");

    let output = command
        .wait_with_output()
        .await
        .expect("Failed to get output from wg pubkey");

    let public_key = String::from_utf8(output.stdout).unwrap().trim().to_string();

    (private_key, public_key)
}

pub async fn generate_wg_config(iface: &InterfaceConf, filename: &str) {
    let mut wg_config = File::create(&filename).await.unwrap();
    let mut output = String::new();

    output.push_str("[Interface]\n");
    writeln!(&mut output, "Address = {}", iface.address).unwrap();
    writeln!(&mut output, "ListenPort = {}", iface.port).unwrap();
    writeln!(&mut output, "PrivateKey = {}", iface.privatekey).unwrap();
    writeln!(&mut output, "PostUp = {}", iface.ifup).unwrap();
    writeln!(&mut output, "PostDown = {}", iface.ifdown).unwrap();

    for peer in &iface.peer {
        output.push_str("\n[Peer]\n");
        writeln!(&mut output, "PublicKey = {}", peer.publickey).unwrap();
        writeln!(&mut output, "AllowedIPs = {}", peer.address).unwrap();
    }

    wg_config.write_all(output.as_bytes()).await.unwrap();
}

pub async fn start(iface: &InterfaceConf) -> Result<(), String> {
    let filename = format!("/tmp/{}.conf", iface.name);
    generate_wg_config(iface, &filename).await;
    let output = Command::new("wg-quick")
        .args(["up", &filename])
        .stderr(Stdio::piped())
        .output()
        .await
        .expect("Failed to run wg-quick");

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8(output.stderr).unwrap())
    }
}

pub async fn stop(iface: &str) -> Result<(), String> {
    let output = Command::new("ip")
        .args(["link", "delete", "dev", iface])
        .stderr(Stdio::piped())
        .output()
        .await
        .expect("Failed to bring down wg interfae");

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8(output.stderr).unwrap())
    }
}

pub async fn refresh(iface: &InterfaceConf) -> Result<(), String> {
    let filename = format!("/tmp/{}.conf", iface.name);
    generate_wg_config(iface, &filename).await;

    let output = Command::new("wg-quick")
        .args(["strip", &filename])
        .output()
        .await
        .unwrap();

    let mut tmp = File::create(&filename).await.unwrap();
    tmp.write_all(&output.stdout).await.unwrap();

    let output = Command::new("wg")
        .args(["syncconf", &iface.name, &filename])
        .output()
        .await
        .unwrap();

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8(output.stderr).unwrap())
    }
}

pub fn generate_peer_config(conf: &InterfaceConf, peer_id: usize) -> String {
    let mut output = String::new();

    let peer = conf.peer.get(peer_id).unwrap();
    writeln!(&mut output, "[Interface]").unwrap();
    writeln!(&mut output, "Address = {}", peer.address).unwrap();
    writeln!(&mut output, "PrivateKey = {}", peer.privatekey).unwrap();

    output.push_str("\n[Peer]\n");
    writeln!(&mut output, "PublicKey = {}", conf.publickey).unwrap();
    writeln!(&mut output, "AllowedIPs = {}", peer.allowedip[0]).unwrap();
    writeln!(&mut output, "Endpoint = 140.238.242.140:{}", conf.port).unwrap();
    output.push_str("PersistentKeepalive = 10");
    output
}
