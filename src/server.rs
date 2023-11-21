use crate::{request::{OperateResult, OperateRequest}, store::kv_error::KvError};

use super::{
    error::{RorError, Result},
    request::{ConnectError, ConnectReply, ConnectRequest, Message, USIZE_SIZE},
    store::kv::DataStore,
    user::{user::User, user_error::UserError},
};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Write,ErrorKind},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time
};

use chrono::Local;
use same_file::is_same_file;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    name: String,
    ip: String,
    port: String,
    data_path: String,
    timeout: u64,
    repl: bool,
    local_user: String,
    default_db: String,
    auto_refresh: u32,
}

impl Config {
    fn default() -> Self {
        Config {
            name: "Default server".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "11451".to_string(),
            data_path: "./data/".to_string(),
            timeout: 300,
            repl: false,
            local_user: String::new(),
            default_db: String::new(),
            auto_refresh: 20,
        }
    }
    pub fn get_server() -> Result<Self> {
        let mut file = File::open("config/server.toml")?;
        let mut c = String::new();
        file.read_to_string(&mut c)?;
        let config = toml::from_str(c.as_str())?;
        Ok(config)
    }
}

pub struct Server {
    config: Config,
    dbs: HashMap<String, Arc<Mutex<DataStore>>>,
}

fn output_prompt<T: std::fmt::Display>(content: T) {
    let time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{0} {1}", time, content)
}

impl Server {
    pub fn new() -> Self {
        let config = match Config::get_server() {
            Ok(config) => config,
            Err(e) => {
                output_prompt(format!("Coult not read configuration file: {0}", e));
                Config::default()
            }
        };
        return Self {
            config,
            dbs: HashMap::new(),
        };
    }

    pub fn start(&mut self) -> Result<()> {
        let addres = format!("{0}:{1}", self.config.ip, &self.config.port);
        let listener = TcpListener::bind(addres.clone())?;
        output_prompt(format!("Server start: {}", addres));
        User::test_file()?;
        loop {
            let (stream, adr) = match listener.accept() {
                Ok(r) => r,
                Err(e) => {
                    output_prompt(format!("Unable to accpet connection from a client: {0}", e));
                    continue;
                }
            };
            output_prompt(format!("New connection: {}", adr));
            match self.handle_connection(stream, adr) {
                Ok(()) => continue,
                Err(e) => output_prompt(format!(
                    "Client [{0}], failed to login, reason: {1}",
                    adr, e
                )),
            }
        }
    }

    fn handle_connection(&mut self, mut stream: TcpStream, address: SocketAddr) -> Result<()> {
        let mut size_buffer = [0 as u8; USIZE_SIZE];
        stream.read_exact(&mut size_buffer)?;
        let size = usize::from_be_bytes(size_buffer);
        let mut head_buffer = vec![0; size];
        stream.read_exact(&mut head_buffer)?;

        let head: ConnectRequest = match bincode::deserialize(&head_buffer) {
            Ok(buf) => buf,
            Err(e) => {
                Self::send_error(stream, ConnectError::RequestError)?;
                return Err(RorError::BincodeError(e));
            }
        };
        let user = match User::login(head.user_name, head.password) {
            Ok(u) => u,
            Err(UserError::UserNotFound(n)) => {
                Self::send_error(stream, ConnectError::UserNotFound)?;
                return Err(RorError::UserError(UserError::UserNotFound(n)));
            }
            Err(UserError::WrongPassWord) => {
                Self::send_error(stream, ConnectError::PasswordError)?;
                return Err(RorError::UserError(UserError::WrongPassWord));
            }
            Err(e) => {
                Self::send_error(stream, ConnectError::ServerError)?;
                return Err(RorError::UserError(e));
            }
        };

        let mut db_path_buf = PathBuf::new();
        db_path_buf.push(&self.config.data_path);
        let db_path = match db_path_buf.into_os_string().into_string() {
            Ok(s) => s,
            Err(_) => {
                Self::send_error(stream, ConnectError::PathError)?;
                return Err(RorError::PathError)?;
            }
        };

        let stream_clone = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                output_prompt(format!("[{0}] The reader cannot be created by the clone method, and the client is disconnected", &address));
                Self::send_error(stream, ConnectError::ServerError)?;
                return Err(RorError::IOError(e));
            }
        };
        let reader = BufReader::new(stream_clone);

        for (key, db) in &self.dbs {
            let exists = match is_same_file(&key, &db_path) {
                Ok(b) => b,
                Err(e) => {
                    Self::send_error(stream, ConnectError::ServerError);
                    return Err(RorError::IOError(e));
                }
            };
        }
        if exists {
            let opened_db = Arc::clone(db);
            if let Err(e) = Self::send_repaly(&mut stream) {
                stream.shutdown(Shutdown::Both)?;
                return Err(e);
            }
            let client = Client {stream, reader, db: opened_db, user, address, timeout: 0, config: self.config.clone(), , set_timeout: self.config.timeout.clone()}
            thread::spawn(|| {client.hadle_client()});
            return Ok(());
        }
        //这行专门用来返回
        Ok(())
    }

    fn send_repaly(stream: &mut TcpStream) -> Result<()> {
        let (buf, _) = Message::new(ConnectReply::Success).as_bytes();
        stream.write(&buf.as_slice())?;
        Ok(())
    }

    fn open_new_db(&mut self, path: String) -> Result<Arc<Mutex<DataStore>>> {
        let db = Arc::new(Mutex::new(DataStore::open(path.as_str())?));
        let arc_clone_db = Arc::clone(&db);
        self.arc_clone_db = Arc::clone(&db);
        self.dbs.insert(path.clone(), db);
        Ok(arc_clone_db)
    }

    fn send_error(mut stream: TcpStream, err: ConnectError) -> Result<()> {
        let (buf, _) = Message::new(err).as_bytes()?;
        stream.write(&buf.as_slice())?;
        stream.shutdown(Shutdown::Both)?;
        Ok(())
    }
}

pub struct Client {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    db: Arc<Mutex<DataStore>>,
    level: String,
    user: User,
    address: SocketAddr,
    timeout: u64,
    config: Config,
    set_timeout: u64,
}

impl Client {
    fn hadle_client(mut self) {
        loop {
            thread::sleep(time::Duration::from_secs(1 as u64));
            if self.timeout >= self.set_timeout {
                output_prompt(format!("Client [{0}] activity timeout", self.address));
                let _ = self.stream.shutdown(Shutdown::Both);
                break;
            }
        }
    }

    fn accept_request(&mut self) -> Result<()> {
        let mut size_buffer = [0 as u8; USIZE_SIZE];
        match self.reader.read_exact(&mut size_buffer) {
            Ok(_) => self.timeout = 0,
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    self.timeout += 1;
                    return Ok(());
                }
                return Err(RorError::IOError(e));
            }
        }
        let body_size = usize::from_be_bytes(size_buffer);
        let mut body_buffer = vec![0 as u8; body_size];
        match self.reader.read(&mut body_buffer) {
            Ok(_) => (),
            Err(_) => return Ok(()),
        }

        let op = match bincode::deserialize(&body_buffer) {
            Ok(r) => r,
            Err(_) => {
                let (buf, _) = Message::new(OperateResult::Failure).as_bytes()?;
                self.stream.write(&buf)?;
                self.stream.flush()?;
                return Ok(());
            }
        };

        match self.match_command(op) {
            Ok(r) => {
                let ( buf, _ ) = Message::new(r).as_bytes()?; 
                self.stream.write(&buf)?;
                self.stream.flush()?;
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    fn match_command(&mut self, command: OperateRequest) -> Result<OperateResult> {
        match command {
            OperateRequest::Get { key } => {
                match self.db.lock().unwrap().get(key) {
                    Ok(v) => return Ok(OperateResult::Found(v)),
                    Err(KvError::KeyNotFound(_)) => return Ok(OperateResult::KeyNotFound),
                    Err(e) => return Err(RorError::KvError(e)),
                }
            }
            OperateRequest::Delete { key } => {
                if self.level != "2" && self.level != "3" {
                    return Ok(OperateResult::PermissionDenied);
                }
                match self.db.lock().unwrap().delete(key) {
                    Ok(_) => {
                        return Ok(OperateResult::Success);
                    }
                    Err(KvError::KeyNotFound(_)) => return Ok(OperateResult::KeyNotFound),
                    Err(e) => return Err(RorError::KvError(e)),
                }
            }
            OperateRequest::Add { key, value } => {
                if self.level != "1" && self.level != "2" && self.level != "3" {
                    return Ok(OperateResult::PermissionDenied);
                }
                match self.db.lock().unwrap().add(key,value) {
                    Ok(_) => {
                        return Ok(OperateResult::Success);
                    }
                    Err(e) => return Err(RorError::KvError(e)),
                }
            }
            OperateRequest::CreateUser { username, password, level } => {
                if self.level != "3" {
                    return Ok(OperateResult::PermissionDenied);
                }
    
                match User::register(                    
                    username,
                    password,
                    &level
                ) {
                    Ok(_) => return Ok(OperateResult::Success),
                    Err(e) => {
                        output_prompt(format!("Unable to create new user for client [{0}], {1}", self.address, e));
                        return Ok(OperateResult::Failure);
                    }
                }
            }
            OperateRequest::DeleteUser { name } => {
                if self.level != "3" {
                    return Ok(OperateResult::PermissionDenied);
                }
    
                match User::delete(name.clone()) {
                    Ok(_) => Ok(OperateResult::Success),
                    Err(e) => {
                        output_prompt(format!("Unable to delete user '{0}' for client [{1}], {2}", name, self.address, e));
                        return Ok(OperateResult::Failure);
                    }
                }
            }
            OperateRequest::GetType { key } => {
                match self.db.lock().unwrap().get(key) {
                    Ok(v) => {
                        return Ok(OperateResult::Type(DataStore::type_of(v)));
                    }
                    Err(KvError::KeyNotFound(_)) => return Ok(OperateResult::KeyNotFound),
                    Err(e) => return Err(RorError::KvError(e)),
                }
            }
            OperateRequest::Compact => {
                if self.level != "1" && self.level != "2" && self.level != "3" {
                    return Ok(OperateResult::PermissionDenied);
                }
                match self.db.lock().unwrap().compact() {
                    Ok(_) => {
                        return Ok(OperateResult::Success);
                    }
                    Err(e) => return Err(RorError::KvError(e)),
                }
            }
            OperateRequest::Quit => {
                return Err(RorError::Disconnect);
            },
        }
    }
}