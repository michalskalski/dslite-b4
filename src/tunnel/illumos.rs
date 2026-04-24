use crate::tunnel::{TunnelBackend, TunnelError};
use std::{
    ffi::{CString, c_char, c_uint, c_void},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

const NI_MAXHOST: usize = 1025;
const DLADM_STATUS_OK: u32 = 0;
const IPADM_STATUS_OK: u32 = 0;
const AF_INET: u16 = 2;

const IPTUN_PARAM_TYPE: c_uint = 0x00000001;
const IPTUN_PARAM_LADDR: c_uint = 0x00000002;
const IPTUN_PARAM_RADDR: c_uint = 0x00000004;
const DLADM_OPT_ACTIVE: c_uint = 0x00000001;
const IPADM_OPT_ACTIVE: u32 = 0x00000002;

unsafe extern "C" {
    fn dladm_open(handle: *mut *mut c_void) -> u32;
    fn dladm_close(handle: *mut c_void);
    fn dladm_iptun_create(
        handle: *mut c_void,
        name: *const c_char,
        params: *mut IpTunParams,
        flags: c_uint,
    ) -> u32;
    fn dladm_iptun_delete(handle: *mut c_void, link_id: u32, flags: c_uint) -> u32;
    fn dladm_name2info(
        handle: *mut c_void,
        name: *const c_char,
        linkid: *mut u32,
        flags: *mut u32,
        class: *mut u32,
        media: *mut u32,
    ) -> u32;
    fn ipadm_open(handle: *mut *mut c_void, flags: u32) -> u32;
    fn ipadm_close(handle: *mut c_void);
    fn ipadm_create_if(handle: *mut c_void, name: *mut c_char, family: u16, flags: u32) -> u32;
    fn ipadm_delete_if(handle: *mut c_void, name: *const c_char, family: u16, flags: u32) -> u32;
}

#[repr(C)]
struct IpsecReq {
    ipsr_ah_req: c_uint,
    ipsr_esp_req: c_uint,
    ipsr_self_encap_req: c_uint,
    ipsr_auth_alg: u8,
    ipsr_esp_alg: u8,
    ipsr_esp_auth_alg: u8,
}

#[repr(u32)]
enum IpTunType {
    Unknown = 0,
    Ipv4 = 1,
    Ipv6 = 2,
    SixToFour = 3,
}

#[repr(C)]
struct IpTunParams {
    link_id: u32,
    flags: c_uint,
    ip_tun_type: IpTunType,
    l_addr: [c_char; NI_MAXHOST],
    r_addr: [c_char; NI_MAXHOST],
    sec_info: IpsecReq,
}

pub struct IllumosBackend {
    cname: CString,
    local_v6: Ipv6Addr,
    remote_v6: Ipv6Addr,
    local_v4: Ipv4Addr,
}

impl IllumosBackend {
    pub fn new(
        name: String,
        local_v6: Ipv6Addr,
        remote_v6: Ipv6Addr,
        local_v4: Ipv4Addr,
    ) -> Result<Self, std::ffi::NulError> {
        let cname = std::ffi::CString::new(name)?;

        Ok(Self {
            cname,
            local_v6,
            remote_v6,
            local_v4,
        })
    }
    fn create_tunnel(&self, handle: &DladmHandle) -> Result<u32, TunnelError> {
        let mut params = build_tunnel_params(&self.local_v6, &self.remote_v6);
        unsafe {
            let status = dladm_iptun_create(
                handle.ptr,
                self.cname.as_ptr(),
                &mut params,
                DLADM_OPT_ACTIVE,
            );
            if status != DLADM_STATUS_OK {
                return Err(TunnelError::CreationFailed(format!(
                    "dladm_iptun_create failed with status {}",
                    status
                )));
            }
        };
        tracing::debug!(link_id = params.link_id, "tunnel created");

        Ok(params.link_id)
    }

    fn create_if(&self, handle: &IpadmHandle) -> Result<(), TunnelError> {
        unsafe {
            let status = ipadm_create_if(
                handle.ptr,
                self.cname.as_ptr() as *mut c_char,
                AF_INET,
                IPADM_OPT_ACTIVE,
            );
            if status != IPADM_STATUS_OK {
                return Err(TunnelError::CreationFailed(format!(
                    "ipadm_create_if failed with status {}",
                    status
                )));
            }
        };
        tracing::debug!("ip interface assigned to tunel");
        Ok(())
    }

    fn delete_tunnel(&self, handle: &DladmHandle) -> Result<(), TunnelError> {
        let (link_id, status) = self.name_to_linkid(handle);
        if status != DLADM_STATUS_OK {
            return Err(TunnelError::DestroyFailed(format!(
                "failed to get linkid, dladm_name2info status: {}",
                status
            )));
        }
        tracing::debug!(link_id, "resolved link id for deletion");
        unsafe {
            let status = dladm_iptun_delete(handle.ptr, link_id, DLADM_OPT_ACTIVE);
            if status != DLADM_STATUS_OK {
                return Err(TunnelError::DestroyFailed(format!(
                    "dladm_iptun_delete failed with status {}",
                    status
                )));
            }
        };

        Ok(())
    }

    fn delete_if(&self, handle: &IpadmHandle) -> Result<(), TunnelError> {
        unsafe {
            let status =
                ipadm_delete_if(handle.ptr, self.cname.as_ptr(), AF_INET, IPADM_OPT_ACTIVE);
            if status != IPADM_STATUS_OK {
                return Err(TunnelError::DestroyFailed(format!(
                    "ipadm_delete_if failed with status {}",
                    status
                )));
            }
        };
        tracing::debug!("ip interface deleted");
        Ok(())
    }

    fn name_to_linkid(&self, handle: &DladmHandle) -> (u32, u32) {
        let mut link_id: u32 = 0;
        let status = unsafe {
            dladm_name2info(
                handle.ptr,
                self.cname.as_ptr(),
                &mut link_id,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        (link_id, status)
    }
}

impl TunnelBackend for IllumosBackend {
    async fn setup(&self) -> Result<(), TunnelError> {
        let handle = open_dladm().map_err(|e| {
            TunnelError::CreationFailed(format!("unable to open handle, dladm_open status {}", e))
        })?;
        let _link_id = self.create_tunnel(&handle)?;

        let ip_handle = open_ipadm().map_err(|e| {
            TunnelError::CreationFailed(format!("unable to open handle, ipadm_open status {}", e))
        })?;
        self.create_if(&ip_handle)?;

        // TODO: assign IPv4 address

        tracing::info!(
            name = %self.cname.to_string_lossy(),
            local_v6 = %self.local_v6,
            remote_v6 = %self.remote_v6,
            "tunnel established"
        );

        Ok(())
    }

    async fn teardown(&self) -> Result<(), TunnelError> {
        let ip_handle = open_ipadm().map_err(|e| {
            TunnelError::DestroyFailed(format!("unable to open handle, ipadm_open status {}", e))
        })?;
        self.delete_if(&ip_handle)?;

        let handle = open_dladm().map_err(|e| {
            TunnelError::DestroyFailed(format!("unable to open handle, dladm_open status {}", e))
        })?;
        self.delete_tunnel(&handle)?;

        tracing::info!(
            name = %self.cname.to_string_lossy(),
            "tunnel removed"
        );

        Ok(())
    }

    async fn is_up(&self) -> Result<bool, TunnelError> {
        Ok(false)
    }
}

struct DladmHandle {
    ptr: *mut c_void,
}

impl Drop for DladmHandle {
    fn drop(&mut self) {
        unsafe { dladm_close(self.ptr) };
    }
}

struct IpadmHandle {
    ptr: *mut c_void,
}

impl Drop for IpadmHandle {
    fn drop(&mut self) {
        unsafe { ipadm_close(self.ptr) }
    }
}

fn addr_to_caddr(addr: &std::net::IpAddr) -> [c_char; NI_MAXHOST] {
    let s = addr.to_string();
    let bytes = s.as_bytes();
    let mut caddr = [0 as c_char; NI_MAXHOST];
    // IPv6 address string is at most 45 bytes, always fits in NI_MAXHOST
    for (i, &b) in bytes.iter().enumerate() {
        caddr[i] = b as c_char;
    }
    caddr
}

fn build_tunnel_params(local: &Ipv6Addr, remote: &Ipv6Addr) -> IpTunParams {
    IpTunParams {
        link_id: 0,
        flags: IPTUN_PARAM_TYPE | IPTUN_PARAM_LADDR | IPTUN_PARAM_RADDR,
        ip_tun_type: IpTunType::Ipv6,
        l_addr: addr_to_caddr(&IpAddr::V6(*local)),
        r_addr: addr_to_caddr(&IpAddr::V6(*remote)),
        sec_info: IpsecReq {
            ipsr_ah_req: 0,
            ipsr_esp_req: 0,
            ipsr_self_encap_req: 0,
            ipsr_auth_alg: 0,
            ipsr_esp_alg: 0,
            ipsr_esp_auth_alg: 0,
        },
    }
}

fn open_dladm() -> Result<DladmHandle, u32> {
    let mut ptr: *mut c_void = std::ptr::null_mut();
    let status = unsafe { dladm_open(&mut ptr) };
    if status != DLADM_STATUS_OK {
        return Err(status);
    }
    Ok(DladmHandle { ptr })
}

fn open_ipadm() -> Result<IpadmHandle, u32> {
    let mut ptr: *mut c_void = std::ptr::null_mut();
    let status = unsafe { ipadm_open(&mut ptr, 0) };
    if status != IPADM_STATUS_OK {
        return Err(status);
    }
    Ok(IpadmHandle { ptr })
}
