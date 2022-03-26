//use interface;
use super::interface;
use std::pin::Pin;
use libc::{c_uchar, size_t};
use std::ffi::CString;
use std::time::Duration;

//OpenVPN MTU
//TODO: query MTU dynamically from OpenVPN C++ code?
const OPENVPN_MTU: usize = 1600;

pub struct OpenVpn {
    instance: *mut interface::OpenVpnInstance,
}

extern "C" fn on_open_vpn_receive(
    instance: Box<OpenVpn>,
    data: *mut c_uchar,
    size: *mut size_t,
) -> u8 {
    /*
    let s = instance.receive(data, size);
    match s {
        Some(_) => 0,
        None => 1,
    }
    */
    0
}

/*
extern "C" fn open_vpn_send(instance: Box<OpenVpn>, data: *const c_uchar, size: size_t) {
    instance.send(data, size);
}
*/
/*
    Attention: if going to use self threaded mode, make sure that
    it panics if using new instead of new_self_threaded, because we
    must call openvpn_set_rust_parent so C++ knows which object to deliver
    the calls to
*/ 
impl OpenVpn {
    pub fn new(profile: &str) -> OpenVpn {
        let profile = CString::new(profile).expect("CString::new failed");
        let instance =  unsafe { interface::openvpn_new(profile.as_ptr()) };
        let o = OpenVpn {
            instance: instance
        }; 
        o
    }

    /*
        In this mode, C++ code has its own thread which writes back to Rust. 
        That's why we use Pin, so it writes back to a real object always.
        TODO: maybe deprecate this? Or at least lave it here.
    */
    pub fn new_self_threaded(profile: &str) -> Pin<Box<OpenVpn>> {
        let profile = CString::new(profile).expect("CString::new failed");
        let instance =  unsafe { interface::openvpn_new(profile.as_ptr()) };
        let o = OpenVpn {
            instance: instance
        };
        let p = Box::pin(o);
        //TODO: Verify if Pin is OK
        unsafe {
            //TODO: fix this line
            //interface::openvpn_set_rust_parent(o.instance, p.as_ptr());
            //interface::openvpn_set_on_send(o.instance, open_vpn_send);
            //interface::openvpn_set_on_receive(o.instance, on_open_vpn_receive);
        };
        p
    }

    pub fn connect(&self) -> Result<(), ()> {
        let r: u8 = unsafe { interface::openvpn_connect(self.instance) };
        match r {
            0 => Ok(()),
            1 => Err(()),
            _ => Err(()),
        }
    }

    pub fn disconnect(&self) -> Result<(), ()> {
        let r: u8 = unsafe { interface::openvpn_disconnect(self.instance) };
        match r {
            0 => Ok(()),
            1 => Err(()),
            _ => Err(()),
        }
    }

    pub fn send(&self, data: *const c_uchar, size: size_t) -> Result<(), ()> {
        let r: u8 = unsafe { interface::openvpn_send(self.instance, data, size) };
        match r {
            0 => Ok(()),
            1 => Err(()),
            _ => Err(()),
        }
    }

    pub fn receive(&self, timeout: Option<Duration>) -> Option<Vec<u8>> {
        let mut array: [u8; OPENVPN_MTU] = [0u8; OPENVPN_MTU];
        let written_size: *mut size_t = std::ptr::null_mut();
        let timeout_millis = match timeout {
            Some(timeout) => timeout.as_millis() as u64,
            None => 0
        };
        let r: u8 = unsafe {
            interface::openvpn_receive(
                self.instance,
                array.as_mut_ptr(),
                array.len(),
                written_size,
                timeout_millis
            )
        };
        match r {
            0 => Some((&array[0..unsafe { *written_size }]).to_vec()),
            1 => None,
            2 => panic!("tried to receive more than array size"),
            _ => panic!("unspported value"),
        }
    }
}

impl Drop for OpenVpn {
    fn drop(&mut self) {
        unsafe { interface::openvpn_destroy(self.instance) };
    }
}
