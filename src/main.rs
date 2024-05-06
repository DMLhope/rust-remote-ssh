use ssh2::Session;
use std::fmt::format;
use std::io::prelude::*;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;

fn execute_ssh_command(ip: &str, username: &str, password: &str, command: &str) {
    // Connect to the local SSH server
    let tcp = TcpStream::connect(format!("{}:22", ip)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password(username, password).unwrap();
    // assert!(sess.authenticated());
    let mut channel = sess.channel_session().unwrap();
    // channel.exec("cd autotests && echo 123 | sudo -S bash ./main.sh").unwrap();
    channel.exec(command).unwrap();

    // 使用线程实时读取命令输出
    let mut buf = [0; 1024];
    loop {
        let nbytes = channel.read(&mut buf).unwrap();
        if nbytes == 0 {
            // 到达输出流的末尾
            break;
        }
        io::stdout().write_all(&buf[..nbytes]).unwrap();
        io::stdout().flush().unwrap();
    }

    // 检查命令执行是否出错
    let mut stderr = channel.stderr();
    let mut err_buf = [0; 1024];
    let err_bytes = stderr.read(&mut err_buf).unwrap();
    if err_bytes > 0 {
        let error_message = String::from_utf8_lossy(&err_buf[..err_bytes]);
        println!("Error executing command: {}", error_message);
    }
    // 等待命令执行完成
    channel.wait_close().unwrap();
    println!("Command execution completed.");
    // println!("{}", channel.exit_status().unwrap());
}

fn upload_file(local_path: &str, remote_path: &str, ip: &str, username: &str, password: &str) {
    // Connect to remote host
    let tcp = TcpStream::connect(format!("{}:22", ip)).unwrap();
    let mut sess = ssh2::Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password(username, password).unwrap();

    // Open a new SFTP session
    let sftp = sess.sftp().unwrap();

    // Create the remote directory if it doesn't exist
    let _ = sftp.mkdir(Path::new(remote_path), 0o755);

    // Iterate over files in local directory
    for entry in std::fs::read_dir(local_path).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let file_path = entry.path();

        // Check if entry is a file
        if file_path.is_file() {
            let mut local_file = File::open(&file_path).unwrap();
            let remote_file_path = Path::new(remote_path).join(&file_name);
            let mut remote_file = sftp.create(&remote_file_path).unwrap();

            // Transfer file content
            let mut buf = [0; 1024];
            loop {
                let nbytes = local_file.read(&mut buf).unwrap();
                if nbytes == 0 {
                    break;
                }
                remote_file.write_all(&buf[..nbytes]).unwrap();
            }
        }
    }
}

fn main() {
    let ip = "192.168.1.191";
    let username = "fxos";
    let password = "123";
    let local_path  = "autotests";
    let remote_path = "/home/fxos/autotests";
    let command = "cd autotests && echo 123 | sudo -S bash ./main.sh";
    upload_file(local_path, remote_path, ip, username, password);
    execute_ssh_command(ip, username, password, command);
}
