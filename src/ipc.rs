use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum IpcCommand {
    OpenFile(PathBuf),
    JumpToHeading(String),
    Reload,
    AskAi(String),
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub success: bool,
    pub message: String,
}

pub struct IpcServer {
    rx: Receiver<IpcCommand>,
    running: Arc<AtomicBool>,
}

impl IpcServer {
    /// Starts local TCP IPC listener on 127.0.0.1:port.
    pub fn start(port: u16, ctx: egui::Context) -> Option<Self> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[IPC] Failed to bind to {}: {}. IPC remote disabled.", addr, e);
                return None;
            }
        };

        // Set non-blocking timeout on listener so it can exit cleanly when shutting down
        let _ = listener.set_nonblocking(true);

        let (tx, rx): (Sender<IpcCommand>, Receiver<IpcCommand>) = mpsc::channel();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
                        let mut reader = BufReader::new(stream.try_clone().unwrap_or_else(|_| stream));
                        let mut line = String::new();

                        if let Ok(n) = reader.read_line(&mut line) {
                            if n > 0 {
                                match serde_json::from_str::<IpcCommand>(line.trim()) {
                                    Ok(cmd) => {
                                        let is_ping = matches!(cmd, IpcCommand::Ping);
                                        let _ = tx.send(cmd);
                                        ctx.request_repaint();

                                        let resp = IpcResponse {
                                            success: true,
                                            message: if is_ping { "pong".to_string() } else { "ok".to_string() },
                                        };
                                        if let Ok(resp_json) = serde_json::to_string(&resp) {
                                            let mut s = reader.into_inner();
                                            let _ = writeln!(s, "{}", resp_json);
                                            let _ = s.flush();
                                        }
                                    }
                                    Err(e) => {
                                        let resp = IpcResponse {
                                            success: false,
                                            message: format!("Invalid command: {}", e),
                                        };
                                        if let Ok(resp_json) = serde_json::to_string(&resp) {
                                            let mut s = reader.into_inner();
                                            let _ = writeln!(s, "{}", resp_json);
                                            let _ = s.flush();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });

        Some(Self { rx, running })
    }

    /// Polls pending commands received over IPC.
    pub fn try_recv(&self) -> Option<IpcCommand> {
        self.rx.try_recv().ok()
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

pub struct IpcClient;

impl IpcClient {
    /// Sends a command to a running mdeader instance on 127.0.0.1:port.
    pub fn send(port: u16, cmd: &IpcCommand) -> Result<String, String> {
        let addr = format!("127.0.0.1:{}", port);
        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| format!("Failed to connect to mdeader at {}: {}. Is mdeader running?", addr, e))?;

        let _ = stream.set_read_timeout(Some(Duration::from_secs(4)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(4)));

        let serialized = serde_json::to_string(cmd)
            .map_err(|e| format!("Failed to serialize command: {}", e))?;

        writeln!(stream, "{}", serialized)
            .map_err(|e| format!("Failed to send command to mdeader: {}", e))?;
        stream.flush().map_err(|e| format!("Flush failed: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .map_err(|e| format!("Failed to read response from mdeader: {}", e))?;

        let resp: IpcResponse = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if resp.success {
            Ok(resp.message)
        } else {
            Err(resp.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_command_serde() {
        let cmd = IpcCommand::OpenFile(PathBuf::from("/tmp/test.md"));
        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: IpcCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, parsed);

        let jump = IpcCommand::JumpToHeading("Section 2".to_string());
        let jump_json = serde_json::to_string(&jump).unwrap();
        let jump_parsed: IpcCommand = serde_json::from_str(&jump_json).unwrap();
        assert_eq!(jump, jump_parsed);
    }

    #[test]
    fn test_ipc_client_server_roundtrip() {
        let test_port = 19898;
        let ctx = egui::Context::default();
        let server = IpcServer::start(test_port, ctx);
        assert!(server.is_some());
        let server = server.unwrap();

        // Give the listener thread a moment to bind
        std::thread::sleep(std::time::Duration::from_millis(50));

        let res = IpcClient::send(test_port, &IpcCommand::Ping);
        assert_eq!(res.unwrap(), "pong");

        let cmd = server.try_recv();
        assert_eq!(cmd, Some(IpcCommand::Ping));
    }
}
