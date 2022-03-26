use super::openvpn2::OpenVpn;
use libc::{c_char, c_int, c_uchar, c_uint, size_t, c_void};
#[repr(C)]
pub struct OpenVpnInstance {
    _private: [u8; 0],
}

type VpnReceive = extern "C" fn(Box<OpenVpn>, *mut c_uchar, *mut size_t) -> u8;
type VpnSend = extern "C" fn(Box<OpenVpn>, *const c_uchar, size_t);

extern "C" {
    pub fn openvpn_new(uri: *const c_char) -> *mut c_void;
    pub fn openvpn_connect(instance: *mut c_void) -> u8;
    pub fn openvpn_disconnect(instance: *mut c_void) -> u8;
    pub fn openvpn_set_rust_parent(instance: *mut c_void, parent: *mut c_void);
    //pub fn openvpn_set_on_receive(instance: *mut OpenVpnInstance, on_receive: VpnReceive);
    pub fn openvpn_send(instance: *mut c_void, data: *const u8, size: size_t) -> u8;
    pub fn openvpn_receive(
        instance: *mut c_void,
        data: *mut u8,
        size: size_t,
        written_size: *mut size_t,
        timeout: u64
    ) -> u8;
    //pub fn openvpn_set_on_send(instance: *mut OpenVpnInstance, on_send: VpnSend);
    pub fn openvpn_init(instance: *mut c_void);
    pub fn openvpn_destroy(instance: *mut c_void);
}
