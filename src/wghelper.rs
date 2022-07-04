use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::{collections::HashSet, fmt::Write};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
    process::Command,
};

const IFUP: &str =
    "iptables -A FORWARD -i %i -j ACCEPT; iptables -t nat -A POSTROUTING -o enp0s3 -j MASQUERADE";
const IFDOWN: &str =
    "iptables -D FORWARD -i %i -j ACCEPT; iptables -t nat -D POSTROUTING -o enp0s3 -j MASQUERADE";

const PATH: &str = "./interfaces.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub name: String,
    pub address: String,
    pub prikey: String,
    pub pubkey: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub path: String,
    pub name: String,
    pub address: String,
    pub subnet: usize,
    pub port: u16,
    pub prikey: String,
    pub pubkey: String,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wg {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub servers: Vec<Server>,
}

impl Wg {
    pub async fn create(&mut self, name: &str, cidr: &str, port: u16) {
        let (prikey, pubkey) = Self::get_keys().await;

        let subnet = cidr.chars().filter(|ch| *ch == 'x').count();
        let subnet = 32 - 8 * subnet;

        let server = Server {
            path: format!("/tmp/{}.conf", name),
            name: name.into(),
            address: cidr.into(),
            subnet,
            port,
            prikey,
            pubkey,
            peers: vec![],
        };

        self.servers.push(server);
    }

    pub async fn start(&self, server_id: usize) -> Result<(), String> {
        self.wg_config(server_id).await;
        let config_file = &self.servers[server_id].path;
        let output = Command::new("wg-quick")
            .args(["up", config_file])
            .output()
            .await
            .unwrap();

        if output.status.success() {
            return Ok(());
        }

        Err(String::from_utf8(output.stderr).unwrap())
    }

    pub async fn stop(&self, server_id: usize) -> Result<(), String> {
        self.wg_config(server_id).await;
        let config_file = &self.servers[server_id].path;
        let output = Command::new("wg-quick")
            .args(["down", config_file])
            .output()
            .await
            .unwrap();

        if output.status.success() {
            return Ok(());
        }

        Err(String::from_utf8(output.stderr).unwrap())
    }

    pub fn read_state() -> Wg {
        let config = std::fs::read_to_string(PATH).unwrap();
        let state: Wg = toml::from_str(&config).unwrap();
        state
    }

    pub async fn dump_state(state: &Wg) {
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

    pub async fn create_peer(&mut self, name: &str, server_id: usize) {
        let (prikey, pubkey) = Self::get_keys().await;
        if let Some(server) = self.servers.get_mut(server_id) {
            let peer = Peer {
                name: name.into(),
                address: format!(
                    "{}/{}",
                    server
                        .address
                        .replace('x', &(server.peers.len() + 2).to_string()),
                    server.subnet
                ),
                prikey,
                pubkey,
                enabled: true,
            };
            server.peers.push(peer);
        }
    }

    pub async fn hot_reload(&self, server_id: usize) {
        if let Some(server) = self.servers.get(server_id) {
            self.wg_config(server_id).await;
            let config_file = &server.path;

            let output = Command::new("wg-quick")
                .args(["strip", config_file])
                .output()
                .await
                .unwrap();

            let output = String::from_utf8(output.stdout).unwrap();

            let update_file_name = format!("/tmp/update_{}.conf", server.name);
            let mut update_file = File::create(&update_file_name).await.unwrap();
            update_file.write_all(output.as_bytes()).await.unwrap();

            let output = Command::new("wg")
                .args(["syncconf", &server.name, &update_file_name])
                .output()
                .await
                .unwrap();

            dbg!(output);
        }
    }

    async fn wg_config(&self, server_id: usize) {
        if let Some(server) = self.servers.get(server_id) {
            let mut file = BufWriter::new(File::create(&server.path).await.unwrap());

            file.write_all(b"[Interface]\n").await.unwrap();
            file.write_all(
                format!(
                    "Address = {}/{}\n",
                    server.address.replace('x', "1"),
                    server.subnet
                )
                .as_bytes(),
            )
            .await
            .unwrap();
            file.write_all(format!("ListenPort = {}\n", server.port).as_bytes())
                .await
                .unwrap();
            file.write_all(format!("PrivateKey = {}\n", server.prikey).as_bytes())
                .await
                .unwrap();
            file.write_all(format!("PostUp = {}\n", IFUP).as_bytes())
                .await
                .unwrap();
            file.write_all(format!("PostDown = {}\n\n", IFDOWN).as_bytes())
                .await
                .unwrap();

            for peer in &server.peers {
                file.write_all(b"[Peer]\n").await.unwrap();
                file.write_all(format!("PublicKey = {}\n", peer.pubkey).as_bytes())
                    .await
                    .unwrap();
                let address: String = peer.address.split('/').take(1).collect();
                file.write_all(format!("AllowedIPs = {}/32\n\n", address).as_bytes())
                    .await
                    .unwrap();
            }

            file.flush().await.unwrap();
        }
    }

    pub fn peer_config(&self, server_id: usize, peer_id: usize) -> String {
        let mut output = String::new();
        if let Some(server) = self.servers.get(server_id) {
            if let Some(peer) = server.peers.get(peer_id) {
                writeln!(&mut output, "[Interface]").unwrap();
                writeln!(&mut output, "Address = {}", peer.address).unwrap();
                writeln!(&mut output, "PrivateKey  = {}\n", peer.prikey).unwrap();
                writeln!(&mut output, "[Peer]").unwrap();
                writeln!(&mut output, "PublicKey = {}", server.pubkey).unwrap();
                writeln!(
                    &mut output,
                    "AllowedIPs = {}/{}",
                    server.address.replace('x', "0"),
                    server.subnet
                )
                .unwrap();
                writeln!(&mut output, "Endpoint = 140.238.242.140:{}", server.port).unwrap();
            }
        }
        output
    }

    pub async fn server_status() -> HashSet<String> {
        let mut status = HashSet::new();
        let output = Command::new("wg")
            .args(["show", "interfaces"])
            .output()
            .await
            .unwrap();

        let output: String = String::from_utf8(output.stdout).unwrap();
        for server in output.trim().split(' ') {
            status.insert(server.into());
        }

        status
    }
}
