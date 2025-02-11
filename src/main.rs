extern crate nix;
use nix::fcntl::{open, OFlag};
use nix::sched::{setns, unshare, CloneFlags};
use nix::{
    sys::stat::Mode,
    mount::{
        umount2, mount, MsFlags, MntFlags
    },
    unistd::{setgid, setuid, Gid, Uid},
};
use std::env;
use std::ffi::CString;

fn main() {
    let nsname = if env::args().len() > 2 {
        env::args().nth(1).unwrap()
    } else {
        panic!("Please supply at least 2 arguments - the network namespace name or the pid of a process whose netns you want to enter, then the command and any arguments to that command");
    };

    // Mainly from https://github.com/iproute2/iproute2/blob/41710ace5e8fadff354f3dba67bf27ed3a3c5ae7/lib/namespace.c#L47
    unshare(CloneFlags::CLONE_NEWNET | CloneFlags::CLONE_NEWNS).expect("Failed to unshare network or mount namespace. Make sure the setuid-bit is set and netns-exec owned by root.");

    let nspath = format!("/var/run/netns/{}", nsname);

    let nsfd = open(nspath.as_str(), OFlag::O_RDONLY, Mode::empty())
        .expect(&format!("Could not open netns file: {}", nspath));

    setns(nsfd, CloneFlags::CLONE_NEWNET).expect("Couldn't set network namespace");

    // make sure mounts don't propagate!
    mount(None::<&str>, "/", None::<&str>, MsFlags::MS_SLAVE | MsFlags::MS_REC, None::<&str>).expect("Could not mount / as ms_slave|ms_rec");

    // build env for non-namespace-aware programs.
    umount2("/sys", MntFlags::MNT_DETACH).expect("sys was not mounted!");
    mount(Some(nsname.as_str()), "/sys", Some("sysfs"), MsFlags::empty(), None::<&str>).expect("Could not mount netns-sys.");

    let ns_etc_nsswitch = format!("/etc/netns/{}/nsswitch.conf", nsname);
    let ns_etc_resolv = format!("/etc/netns/{}/resolv.conf", nsname);

    mount(Some(ns_etc_nsswitch.as_str()), "/etc/nsswitch.conf", None::<&str>, MsFlags::MS_PRIVATE | MsFlags::MS_BIND, None::<&str>).expect(format!("Could not mount {}", ns_etc_nsswitch).as_str());
    mount(Some(ns_etc_resolv.as_str()), "/etc/resolv.conf", None::<&str>, MsFlags::MS_PRIVATE | MsFlags::MS_BIND, None::<&str>).expect(format!("Could not mount {}", ns_etc_nsswitch).as_str());

    mount(Some("/var/empty"), "/var/run/nscd", None::<&str>, MsFlags::MS_PRIVATE | MsFlags::MS_BIND, None::<&str>).expect("Could not bind-mount empty directory onto /var/run/nscd.");

    // drop privs now - these MUST happen in the below order, otherwise
    // dropping group privileges might fail as the user privs may have
    // changed so that the user can no longer set the gid
    setgid(Gid::current()).expect("Couldn't drop group privileges");
    setuid(Uid::current()).expect("Couldn't drop user privileges");

    let args: Vec<_> = env::args()
        .into_iter()
        .skip(2)
        .map(|arg| CString::new(arg.as_str()).unwrap())
        .collect();

    let c_args: Vec<_> = args.iter().map(|arg| arg.as_c_str()).collect();

    nix::unistd::execvp(&c_args.first().unwrap(), c_args.as_slice())
        .expect("something went wrong executing the given command, perhaps it couldn't be found?");
}
