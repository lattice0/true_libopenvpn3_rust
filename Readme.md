# True OpenVPN library Rust

This is a Rust interface to https://github.com/lattice0/true_libopenvpn3. I need to review the C++ interface, and maybe switch for usage with the CXX crate. PRs are appreciated!

For a full example putting everything together to do an HTTP request over userspace OpenVPN, check https://github.com/lattice0/hyper_vpn

# Why?

This library is useful because you don't need privileged capabilities to create/access tun/tap interfaces, so you can support OpenVPN connections on your app on Android for example without requiring VPN permissions. Also, you can connect to multiple OpenVPN servers through multiple profiles and send packets through them on Android, where traditionally it would let you have just one connection at the same time.

# TODO
- use https://github.com/dtolnay/cxx instead of handwritten C++ interface
- clean lots of stuff
