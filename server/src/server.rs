use crate::connection::Connection;
use futures_util::StreamExt;
use quinn::{Endpoint, EndpointConfig, Incoming, ServerConfig};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    io::Error as IoError,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    time::Duration,
};

pub struct Server {
    incoming: Incoming,
    port: u16,
    expected_token_digest: [u8; 32],
    authentication_timeout: Duration,
    max_udp_packet_size: usize,
}

impl Server {
    pub fn init(
        config: ServerConfig,
        port: u16,
        exp_tkn_dgst: [u8; 32],
        auth_timeout: Duration,
        max_udp_pkt_size: usize,
        enable_ipv6: bool,
    ) -> Result<Self, IoError> {
        let (addr, domain) = if enable_ipv6 {
            (
                SocketAddr::from((Ipv6Addr::UNSPECIFIED, port)),
                Domain::IPV6,
            )
        } else {
            (
                SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)),
                Domain::IPV4,
            )
        };

        let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;

        if enable_ipv6 {
            socket.set_only_v6(false)?;
        }

        socket.bind(&SockAddr::from(addr))?;
        let socket = UdpSocket::from(socket);

        let (_, incoming) = Endpoint::new(EndpointConfig::default(), Some(config), socket)?;

        Ok(Self {
            incoming,
            port,
            expected_token_digest: exp_tkn_dgst,
            authentication_timeout: auth_timeout,
            max_udp_packet_size: max_udp_pkt_size,
        })
    }

    pub async fn run(mut self) {
        log::info!("Server started. Listening port: {}", self.port);

        while let Some(conn) = self.incoming.next().await {
            tokio::spawn(Connection::handle(
                conn,
                self.expected_token_digest,
                self.authentication_timeout,
                self.max_udp_packet_size,
            ));
        }
    }
}
