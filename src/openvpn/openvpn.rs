use std::io::Result;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
//use std::collections::VecDeque;
//use core::task::Waker;
use libc::{c_int, c_void, size_t, c_char};
use std::string::String;
use std::time::Duration;
use simple_vpn::{VpnClient, VpnConnectionError, VpnDisconnectionError, PhySendError, PhyReceiveError};

const MAX_BYTES_TRANSPORT: usize = 1518;

///Bridge between Rust and OpenVpn3's C++ (wrapped) library

//TODO: deprecate?
pub type OnVpnRead = Arc<dyn Fn() -> Option<Vec<u8>> + Send + Sync>;
//TODO: deprecate?
pub type OnVpnWrite = Arc<dyn Fn(&[u8]) -> Result<()> + Send + Sync>;
pub type OnVpnLog = Arc<Mutex<dyn Fn(String) + Send + Sync>>;
pub type OnVpnEvent = Arc<Mutex<dyn Fn(OVPNEvent) + Send + Sync>>;

pub struct OVPNClient {
    openvpn_client: *mut c_void,
}

#[derive(Debug)]
pub struct OVPNEvent {
    pub name: String,
    pub info: String,
    error: bool,
    fatal: bool
}

impl std::fmt::Display for OVPNEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if !self.error {
            if self.info.is_empty() {
                write!(f, "E: {}", self.name)
            } else {
                write!(f, "E: {}, I: {}", self.name, self.info)
            }
        } else {
            if self.info.is_empty() {
                write!(f, "E: {}, I: {}, error: {}, fatal: {}", self.name, self.info, self.error, self.fatal)

            } else {
                write!(f, "E: {}, error: {}, fatal: {}", self.name, self.error, self.fatal)
            }
        }
    }
}

pub enum OpenVpnReceiveError {
    NoDataAvailable,
    Unknown(String)
}
pub enum OpenVpnSendError {
    Unknown(String)
}
pub enum OpenVpnConnectionError {
    Unknown(String)
}
pub enum OpenVpnDisconnectionError {
    Unknown(String)
}

impl From<OpenVpnReceiveError> for PhyReceiveError {
    fn from(e: OpenVpnReceiveError) -> PhyReceiveError {
        match e {
            OpenVpnReceiveError::NoDataAvailable => PhyReceiveError::NoDataAvailable,
            OpenVpnReceiveError::Unknown(s) => PhyReceiveError::Unknown(s)
        }
    }
}

impl From<OpenVpnSendError> for PhySendError {
    fn from(e: OpenVpnSendError) -> PhySendError {
        match e {
            OpenVpnSendError::Unknown(s) => PhySendError::Unknown(s)
        }
    }
}

impl From<OpenVpnConnectionError> for VpnConnectionError {
    fn from(e: OpenVpnConnectionError) -> VpnConnectionError {
        match e {
            OpenVpnConnectionError::Unknown(s) => VpnConnectionError::Unknown(s)
        }
    }
}

impl From<OpenVpnDisconnectionError> for VpnDisconnectionError {
    fn from(e: OpenVpnDisconnectionError) -> VpnDisconnectionError {
        match e {
            OpenVpnDisconnectionError::Unknown(s) => VpnDisconnectionError::Unknown(s)
        }
    }
}

unsafe impl Send for OVPNClient {}

struct OVPNClientInner {
    on_vpn_read: Option<OnVpnRead>,
    on_vpn_write: Option<OnVpnWrite>,
    on_vpn_log: Option<OnVpnLog>,
    on_vpn_event: Option<OnVpnEvent>,
    //replacement_ip: String
}

impl OVPNClientInner {
    //Gets data from Rust through on_vpn_read and passes to C++
    fn read_allocate(&mut self, buffer: *mut *mut u8) -> Result<usize> {
        let vpn_buffer = (self.on_vpn_read.as_ref().unwrap())();
        match vpn_buffer {
            Some(vpn_buffer) => {
                let s = vpn_buffer.len();
                let b: *mut u8 = unsafe{openvpn_client_allocate(s as usize)};
                //fill buffer
                unsafe{
                    for i in 0..s {
                        *b.offset(i as isize) = vpn_buffer[i];
                    }
                    *buffer = b
                };
                Ok(s as usize)
            },
            None => {
                Err(std::io::Error::from_raw_os_error(1))
            }
        }
    }

    //Writes data from C++ to Rust
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (self.on_vpn_write.as_ref().unwrap())(buf).unwrap();
        Ok(buf.len())
    }

    //Writes data from C++ to Rust
    fn log(&mut self, c_buffer: *const c_char) -> Result<()> {
        let c_str: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(c_buffer) };
        let str_slice: &str = c_str.to_str().unwrap();
        let str_buf: String = str_slice.to_owned();  
        match self.on_vpn_log.clone() {
            Some(on_vpn_log) => {
                (on_vpn_log.lock().unwrap())(str_buf);
            },
            None => {
                println!("OpenVPN: {}", str_buf);
            }
        }
        Ok(())
    }

    //Writes data from C++ to Rust
    fn event(&mut self, name: *const c_char, info: *const c_char, error: bool, fatal: bool) -> Result<()> {
        let str = |c_buffer: *const c_char|->String {
            let c_str: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(c_buffer) };
            let str_slice: &str = c_str.to_str().unwrap();
            let str_buf: String = str_slice.to_owned();  
            str_buf
        };
        let event = OVPNEvent{
            name: str(name),
            info: str(info),
            error: error,
            fatal: fatal
        };
        match self.on_vpn_event.clone() {
            Some(on_vpn_event) => {
                (on_vpn_event.lock().unwrap())(event);
            },
            None => {
                println!("EVENT: {}", event);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum OVPNCreationError {
    CStringError(String)
}

impl VpnClient for OVPNClient {
    fn set_username(&mut self, _s: Option<&str>) {

    }
    fn set_password(&mut self, _s: Option<&str>) {

    }
    //fn set_vpn_connect(&mut self, f: VpnConnect);
    //fn set_vpn_disconnect(&mut self, f: VpnDisconnect);
    fn vpn_connect(&mut self) -> std::result::Result<(), VpnConnectionError> {
        Ok(())
    }
    fn vpn_disconnect(&mut self) -> std::result::Result<(), VpnDisconnectionError> {
        Ok(())
    }
    //fn set_phy_send(&mut self, f: PhySend);
    //fn set_phy_receive(&mut self, f: PhyReceive);
    fn phy_receive(&mut self, _timeout: Option<Duration>, f: &mut dyn FnMut(&[u8])) -> std::result::Result<usize, PhyReceiveError> {
        self.receive(f).map_err(|e|e.into())
        //Ok(())
    }
    //TODO: change Err return
    fn phy_send(&self, vpn_packet: &[u8]) -> std::result::Result<usize, PhySendError> {
        self.send(vpn_packet).map_err(|e|e.into())
        //Ok(())
    }
}

impl OVPNClient {
    pub fn new(profile: String, 
        username: Option<&str>, 
        password: Option<&str>, 
        on_vpn_read: Option<OnVpnRead>, 
        on_vpn_write: Option<OnVpnWrite>,
        on_vpn_log: Option<OnVpnLog>,
        on_vpn_event: Option<OnVpnEvent>,
        replacement_ipv4: &std::net::Ipv4Addr,
        replacement_ipv6: &std::net::Ipv6Addr) -> std::result::Result<OVPNClient, OVPNCreationError> {
        let inner = OVPNClientInner{
            on_vpn_read: on_vpn_read,
            on_vpn_write: on_vpn_write,
            on_vpn_log: on_vpn_log,
            on_vpn_event: on_vpn_event
        };
        let callbacks = Callbacks {
            user_data: Box::into_raw(Box::new(inner)) as *mut c_void,
            on_read_allocate: on_read_allocate_trampoline,
            on_write: on_write_trampoline,
            on_log: on_log_trampoline,
            on_event: on_event_trampoline,
            destroy: destroy_trampoline::<OVPNClientInner>,
        };
        let profile_cstring = CString::new(profile).map_err(|_|OVPNCreationError::CStringError("CString::new failed for profile_cstring".into()))?;
        let username_cstring = CString::new(username.unwrap_or("")).map_err(|_|OVPNCreationError::CStringError("CString::new failed for username_cstring".into()))?;
        let password_cstring = CString::new(password.unwrap_or("")).map_err(|_|OVPNCreationError::CStringError("CString::new failed for password_cstring".into()))?;
        let replacement_ipv4_addr_string:&str = &replacement_ipv4.to_string();
        let replacement_ipv6_addr_string:&str = &replacement_ipv6.to_string();
        let replacement_ipv4_cstring = CString::new(replacement_ipv4_addr_string).map_err(|_|OVPNCreationError::CStringError("CString::new failed for replacementIpv4".into()))?;
        let replacement_ipv6_cstring = CString::new(replacement_ipv6_addr_string).map_err(|_|OVPNCreationError::CStringError("CString::new failed for replacementIpv6".into()))?;

        Ok(OVPNClient {
            openvpn_client: unsafe{openvpn_client_new((&profile_cstring).as_ptr(), (&username_cstring).as_ptr(), (&password_cstring).as_ptr(), callbacks, (&replacement_ipv4_cstring).as_ptr(), (&replacement_ipv6_cstring).as_ptr())},
        })
    }

    //Deprecated
    pub fn run(&self) -> std::result::Result<(), ()>  {
        let r = unsafe{openvpn_client_run(self.openvpn_client)};
        if r==0 {
            Ok(())
        } else {
            Err(())
        }
    }

    // Sends data to the VPN
    pub fn send(&self, data: &[u8]) -> std::result::Result<usize, OpenVpnSendError> {
        let size = data.len();
        let r = unsafe{openvpn_client_send(data.as_ptr(), size, self.openvpn_client)};
        if r==0 {
            //we always return the full size because the C++ openvpn implementation is always able to receive the full size
            Ok(size)
        } else {
            Err(OpenVpnSendError::Unknown(format!("openvpn send unknown error: {}", r)))
        }
    }

    pub fn connect(&self) -> std::result::Result<(), OpenVpnConnectionError> {
        let r = unsafe{openvpn_client_connect(self.openvpn_client)};
        if r==0 {
            Ok(())
        } else {
            Err(OpenVpnConnectionError::Unknown("".into()))
        }
    }

    pub fn disconnect(&self) -> std::result::Result<(), OpenVpnDisconnectionError>{
        let r = unsafe{openvpn_client_disconnect(self.openvpn_client)};
        if r==0 {
            Ok(())
        } else {
            Err(OpenVpnDisconnectionError::Unknown("".into()))
        }
    }

    // Receives data from the VPN
    pub fn receive(&mut self, f: &mut dyn FnMut(&[u8])) -> std::result::Result<usize, OpenVpnReceiveError> {
        //TODO: https://en.wikipedia.org/wiki/Jumbo_frame
        let mut buffer = [0u8; MAX_BYTES_TRANSPORT];
        //number of bytes written in buffer in the C++ side
        let mut written_size: size_t = 0;
        let r = unsafe{openvpn_client_receive_just(buffer.as_mut_ptr(), buffer.len(), &mut written_size, self.openvpn_client)};
        if r==0 {
            let buffer_slice = &buffer[0..written_size];
            f(buffer_slice);
            Ok(written_size)
        } else if r==2 {
            //Error 2 means there was no data avaliable at the time
            Err(OpenVpnReceiveError::NoDataAvailable)
        } else {
            Err(OpenVpnReceiveError::Unknown("openvpn_client_receive_just unknown error (we shouldn't have arrived here)".to_string()))
        }
    }
    
}

impl Drop for OVPNClient {
    fn drop(&mut self) {
        unsafe{openvpn_client_free(self.openvpn_client)};
    }
}

extern "C" {
    /// Creates a new OpenVPN C++ client, giving it ownership of the object
    /// inside [`Callbacks`].
    fn openvpn_client_new(profile: *const c_char, username: *const c_char, password: *const c_char, callbacks: Callbacks, replacementIpv4: *const c_char, replacementIpv6: *const c_char) -> *mut OpenVpnClient;
    /// Sends data to the VPN
    fn openvpn_client_send(buffer: *const u8, size: size_t, client: *mut OpenVpnClient) -> u8;
    /// Receives data from the VPN
    //fn openvpn_client_receive(buffer: *mut u8, buffer_size: size_t, written_size: *mut size_t, client: *mut OpenVpnClient) -> u8;
    /// Receives data from the VPN, reading just buffer_size from the client
    fn openvpn_client_receive_just(buffer: *mut u8, buffer_size: size_t, written_size: *mut size_t, client: *mut OpenVpnClient) -> u8;
    /// Launches the connect thread of openvpn
    fn openvpn_client_connect(client: *mut OpenVpnClient) -> u8;
    /// Disconnects the connect threaf of openvpn
    fn openvpn_client_disconnect(client: *mut OpenVpnClient) -> u8;
    /// Tell the OpenVPN client to keep running until the VPN is shut down.
    fn openvpn_client_run(client: *mut OpenVpnClient) -> u8;
    /// Destroy the OpenVPN client.
    fn openvpn_client_free(client: *mut OpenVpnClient);
    /// Allocates, on C++, a uint8_t* buffer with size `size`
    fn openvpn_client_allocate(size: size_t) -> *mut u8;
    // Deallocates, on C++, a uint8_t* buffer
    //fn openvpn_client_deallocate(buffer: *mut u8);
}

/// An opaque type representing the C++ OpenVPN client.
type OpenVpnClient = c_void;

#[repr(C)]
pub struct Callbacks {
    /// A pointer to some user-defined state.
    pub user_data: *mut c_void,
    /// Callback fired when the OpenVPN client wants to read data but does not know the data size
    /// so it leaves to Rust the task of allocating. Returns 0 on success, -1 on failure
    pub on_read_allocate: unsafe extern "C" fn(*mut *mut u8, *mut size_t, *mut c_void) -> c_int,
    /// Callback fired when the OpenVPN client wants to write some data.
    pub on_write: unsafe extern "C" fn(*const u8, size_t, *mut c_void) -> c_int,
    /// Callback fired when the OpenVPN client wants to log some data.
    pub on_log: unsafe extern "C" fn(*const c_char, *mut c_void) -> c_int,
    /// Callback fired when the OpenVPN client sends some OpenvVPN event
    pub on_event: unsafe extern "C" fn(*const c_char, *const c_char, bool, bool, *mut c_void) -> c_int,
    /// A function for destroying the user-defined state.
    pub destroy: unsafe extern "C" fn(*mut c_void),
}

unsafe extern "C" fn on_read_allocate_trampoline(
    buffer: *mut *mut u8,
    len: *mut size_t,
    user_data: *mut c_void,
) -> c_int {
    let ovpn_client_inner = &mut *(user_data as *mut OVPNClientInner);
    match ovpn_client_inner.read_allocate(buffer) {
        Ok(allocated_size) => {
            *len = allocated_size;
            0 as c_int
        },
        Err(_) => {
            -1
        },
    }
}

unsafe extern "C" fn on_write_trampoline(
    buffer: *const u8,
    len: size_t,
    user_data: *mut c_void,
) -> c_int {
    let ovpn_client_inner = &mut *(user_data as *mut OVPNClientInner);
    let buffer = std::slice::from_raw_parts(buffer as *const u8, len as usize);

    match ovpn_client_inner.write(buffer) {
        Ok(bytes_written) => bytes_written as c_int,
        Err(_) => -1,
    }
}


unsafe extern "C" fn on_log_trampoline(
    buffer: *const c_char,
    user_data: *mut c_void,
) -> c_int {
    let ovpn_client_inner = &mut *(user_data as *mut OVPNClientInner);
    //let buffer = std::slice::from_raw_parts(buffer as *const u8, len as usize);

    match ovpn_client_inner.log(buffer) {
        Ok(_) => 0,
        Err(_) => 1
    }
}

unsafe extern "C" fn on_event_trampoline(
    name: *const c_char,
    info: *const c_char,
    error: bool,
    fatal: bool,
    user_data: *mut c_void,
) -> c_int {
    let ovpn_client_inner = &mut *(user_data as *mut OVPNClientInner);
    //let buffer = std::slice::from_raw_parts(buffer as *const u8, len as usize);

    match ovpn_client_inner.event(name, info, error, fatal) {
        Ok(_) => 0,
        Err(_) => 1
    }
}

unsafe extern "C" fn destroy_trampoline<P>(user_data: *mut c_void) {
    let user_data = Box::from_raw(user_data as *mut P);
    drop(user_data);
}