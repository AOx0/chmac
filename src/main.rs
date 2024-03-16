#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![deny(clippy::unwrap_used)]

use libc::{
    __c_anonymous_ifr_ifru, c_char, close, ifconf, ifreq, ioctl, sockaddr, socket, AF_INET,
    ARPHRD_ETHER, IFF_UP, IF_NAMESIZE, SIOCETHTOOL, SIOCGIFCONF, SIOCGIFFLAGS, SIOCGIFHWADDR,
    SIOCSIFFLAGS, SIOCSIFHWADDR, SOCK_DGRAM,
};
use mac::Mac;
use std::io::Write;
use thiserror::Error as TError;

mod args;
mod mac;
use args::{Args, Command, Parser};

/// Defined at `/usr/include/linux/ethtool.h`
const ETHTOOL_GPERMADDR: u32 = 0x0000_0020;

fn set_flags(s: i32, devi: &str, set: i32, clr: i32) -> Result<(), Error> {
    set_flags_i(
        s,
        devi,
        set.try_into().map_err(|_| Error::CantGetFlagFromValue(s))?,
        clr.try_into().map_err(|_| Error::CantGetFlagFromValue(s))?,
    )
}

fn set_flags_i(s: i32, devi: &str, set: i16, clr: i16) -> Result<(), Error> {
    let mut req: ifreq = unsafe { std::mem::zeroed() };

    req.ifr_name = get_dev_buff(devi);
    req.ifr_ifru.ifru_flags = get_flags(s, devi)? & (!clr) | set;
    if unsafe { ioctl(s, SIOCSIFFLAGS, std::ptr::from_ref::<ifreq>(&req)) } < 0 {
        return Err(Error::CantSetIfFlag(devi.to_string()));
    }

    Ok(())
}

#[derive(Debug, TError)]
enum Error {
    #[error("MacError: {0}")]
    Mac(#[from] mac::Invalid),
    #[error("There was an error getting AF_INET socket")]
    CantGetSocket,
    #[error("There was an error setting the interface {0:#?} flags")]
    CantSetIfFlag(String),
    #[error("There was an error getting the interface {0:#?} flags")]
    CantGetIfFlag(String),
    #[error("There was an error setting the interface {0:#?} MAC address")]
    CantSetMacAddr(String),
    #[error("There was an error getting the interface {0:#?} MAC address")]
    CantGetMacAddr(String),
    #[error("There was an error getting the interface {0:#?} permanent MAC address")]
    CantGetPermMacAddr(String),
    #[error("There was an error transforming i32 value {0:?} ({0:0>32b}) flag to i16")]
    CantGetFlagFromValue(i32),
    #[error("There was an error reading all interfaces")]
    CantGetInterfaces,
}

#[allow(dead_code)]
fn get_mac(sock: i32, devi: &str) -> Result<Mac, Error> {
    let mut req: ifreq = unsafe { std::mem::zeroed() };
    req.ifr_name = get_dev_buff(devi);

    if unsafe { ioctl(sock, SIOCGIFHWADDR, std::ptr::from_ref::<ifreq>(&req)) } < 0 {
        Err(Error::CantGetMacAddr(devi.to_string()))
    } else {
        unsafe { Ok(Mac(req.ifr_ifru.ifru_hwaddr.sa_data)) }
    }
}

/// Defined at `/usr/include/linux/ethtool.h`
#[repr(C)]
struct EthtoolPermAddr {
    cmd: u32,
    size: u32,
    data: [i8; 14],
}

fn get_perm_mac(sock: i32, devi: &str) -> Result<Mac, Error> {
    let mut req: ifreq = unsafe { std::mem::zeroed() };

    let mut addr_req = EthtoolPermAddr {
        cmd: ETHTOOL_GPERMADDR,
        size: 14_u32,
        data: [0i8; 14_usize],
    };

    req.ifr_name = get_dev_buff(devi);
    req.ifr_ifru.ifru_data = std::ptr::from_mut(&mut addr_req).cast::<i8>();

    if unsafe { ioctl(sock, SIOCETHTOOL, &mut req) } < 0 {
        Err(Error::CantGetPermMacAddr(devi.to_string()))
    } else {
        let data: *mut EthtoolPermAddr = unsafe { req.ifr_ifru.ifru_data }.cast();
        unsafe { Ok(Mac((*data).data)) }
    }
}

fn set_mac_addr(sock: i32, devi: &str, mac: Mac) -> Result<(), Error> {
    let mut req: ifreq = unsafe { std::mem::zeroed() };
    req.ifr_name = get_dev_buff(devi);
    req.ifr_ifru = __c_anonymous_ifr_ifru {
        ifru_hwaddr: sockaddr {
            sa_family: ARPHRD_ETHER,
            sa_data: mac.bytes(),
        },
    };

    if unsafe { ioctl(sock, SIOCSIFHWADDR, std::ptr::from_ref::<ifreq>(&req)) } == -1 {
        return Err(Error::CantSetMacAddr(devi.to_string()));
    }

    Ok(())
}

fn get_dev_buff(devi: &str) -> [i8; 16] {
    let mut ifr_name: [c_char; IF_NAMESIZE] = [0; IF_NAMESIZE];
    ifr_name[..devi.bytes().len()]
        .copy_from_slice(unsafe { &*(std::ptr::from_ref::<str>(devi) as *const [i8]) });
    ifr_name
}

fn get_flags(s: i32, devi: &str) -> Result<i16, Error> {
    let req = ifreq {
        ifr_name: get_dev_buff(devi),
        ifr_ifru: unsafe { std::mem::zeroed() },
    };

    if unsafe { ioctl(s, SIOCGIFFLAGS, std::ptr::from_ref::<ifreq>(&req)) } == -1 {
        return Err(Error::CantGetIfFlag(devi.to_string()));
    }

    Ok(unsafe { req.ifr_ifru.ifru_flags })
}

fn get_socket() -> Result<i32, Error> {
    let sock = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
    if sock < 0 {
        return Err(Error::CantGetSocket);
    }
    Ok(sock)
}

fn main() {
    let args = Args::parse();
    let s = match get_socket() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };

    if let Err(e) = app(s, &args) {
        eprintln!("Error: {e}");
    };

    if unsafe { close(s) } < 0 {
        eprintln!("Failed to close {s}");
    }
}

fn app(s: i32, args: &Args) -> Result<(), Error> {
    match &args.command {
        Command::Reset { ifname } => {
            let mac = get_perm_mac(s, ifname)?;
            set_flags(s, ifname, 0, IFF_UP)?;
            let res = set_mac_addr(s, ifname, mac);
            set_flags(s, ifname, IFF_UP, 0)?;

            if res.is_ok() {
                println!("{mac}");
            }

            res
        }
        Command::Set { ifname, addr } => {
            let mac: Mac = addr.as_str().try_into().map_err(Error::Mac)?;
            set_flags(s, ifname, 0, IFF_UP)?;
            let res = set_mac_addr(s, ifname, mac);
            set_flags(s, ifname, IFF_UP, 0)?;

            if res.is_ok() {
                println!("{mac}");
            }

            res
        }
        Command::Random { ifname } => {
            let mac: Mac = Mac::rand();
            set_flags(s, ifname, 0, IFF_UP)?;
            let res = set_mac_addr(s, ifname, mac);
            set_flags(s, ifname, IFF_UP, 0)?;

            if res.is_ok() {
                println!("{mac}");
            }

            res
        }
        Command::Get { ifname } => {
            let mac = get_mac(s, ifname)?;

            println!("{mac}");
            Ok(())
        }
        Command::Perm { ifname } => {
            let mac = get_perm_mac(s, ifname)?;

            println!("{mac}");
            Ok(())
        }
        &Command::Inames { single_line } => get_ifnames(s, single_line),
        Command::Completions { shell } => {
            match shell {
                args::Shell::Fish => println!(
                    "{}",
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/completions/chmac.fish"
                    ))
                ),
            };

            Ok(())
        }
    }
}

fn get_ifnames(s: i32, single_line: bool) -> Result<(), Error> {
    // First we get the number of necessary bytes
    let mut ifc: ifconf = unsafe { std::mem::zeroed() };
    if unsafe { ioctl(s, SIOCGIFCONF, &mut ifc) } < 0 {
        return Err(Error::CantGetInterfaces);
    }

    // Allocate the necessary bytes & request full info
    let len: usize = ifc
        .ifc_len
        .try_into()
        .expect("This should never be negative");
    let mut buff = vec![0u8; len];
    ifc.ifc_ifcu.ifcu_buf = buff.as_mut_ptr().cast::<i8>();
    if unsafe { ioctl(s, SIOCGIFCONF, &mut ifc) } < 0 {
        return Err(Error::CantGetInterfaces);
    }

    // Transmute as slice of `ifreq`
    let n = len / std::mem::size_of::<ifreq>();
    let ifreqs: &[ifreq] = unsafe { std::slice::from_raw_parts(buff.as_mut_ptr().cast(), n) };

    // Print all if names
    for (i, ifreq) in ifreqs.iter().enumerate() {
        let name: [u8; 16] = unsafe { std::mem::transmute(ifreq.ifr_name) };
        let needle = name.iter().position(|a| a == &0).unwrap_or(16);
        let str = std::str::from_utf8(&name[..needle]).unwrap_or("err");

        if single_line {
            print!("{str}");
            if i + 1 < n {
                print!(" ");
            }
        } else {
            println!("{str}");
        }
    }

    if single_line {
        print!("\n");
        let _ = std::io::stdout().flush();
    }

    Ok(())
}
