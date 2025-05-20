/// Packet delivery [Quality of Service] level.
///
/// [Quality of Service]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099
#[derive(Clone, Debug)]
pub enum QoS {
    /// `QoS 0`. At most once. No ack needed.
    Level0 = 0,
    /// `QoS 1`. At least once. One ack needed.
    Level1 = 1,
    /// `QoS 2`. Exactly once. Two acks needed.
    Level2 = 2,
}

/// ClientId is the internal id assigned to the client by the server, u64 will never overflow, so we
/// can safely assume this is unique
pub type ClientId = u64;