use libp2p::{Swarm, gossipsub::{Gossipsub, GossipsubConfigBuilder}, kad::Kademlia, identity, PeerId};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum BitmapMessage {
    BitmapRegistration { entry: super::database::BitmapEntry },
    BitmapTransfer { blockheight: String, new_owner: String, transfer_block: u64 },
    BnsInscription { entry: super::database::BnsEntry },
    TimestampRequest { inscription_id: String },
    TimestampResponse { inscription_id: String, timestamp: u64 },
}

pub struct Network {
    swarm: Swarm<Gossipsub>,
}

impl Network {
    const BITMAP_TOPIC: &'static str = "bitmap";
    pub fn new(local_key: identity::Keypair, bootstrap_nodes: Vec<String>, network: &str) -> Self {
        let local_peer_id = PeerId::from(local_key.public());
        let gossipsub = Gossipsub::new(local_peer_id, GossipsubConfigBuilder::default().build().unwrap()).unwrap();
        let mut swarm = Swarm::new(
            libp2p::tcp::GenTcpConfig::new(),
            gossipsub,
            Kademlia::new(local_peer_id, libp2p::kad::store::MemoryStore::new(local_peer_id)),
        );
        // Subscribe to a single unified topic for all bitmap and BNS traffic
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&Self::BITMAP_TOPIC.as_bytes().to_vec())
            .unwrap();
        for node in bootstrap_nodes {
            swarm.dial(node.parse().unwrap()).unwrap();
        }
        Network { swarm }
    }

    pub fn broadcast_message(&mut self, message: &BitmapMessage) {
        let message_bytes = serde_cbor::to_vec(message).unwrap();
        // Publish to the single unified topic
        self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(Self::BITMAP_TOPIC.as_bytes().to_vec(), message_bytes)
            .unwrap();
    }

    pub fn poll(&mut self) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            loop {
                if let Some(event) = self.swarm.next().await {
                    // Handle incoming messages, sync registries
                }
            }
        });
    }

    // Explicit helper wrappers so callers can invoke clear domain-specific commands
    // All messages are still sent on the single "bitmap" topic.
    pub fn send_bitmap_register(&mut self, entry: &super::database::BitmapEntry) {
        self.broadcast_message(&BitmapMessage::BitmapRegistration { entry: entry.clone() });
    }

    pub fn send_bitmap_transfer(
        &mut self,
        blockheight: &str,
        new_owner: &str,
        transfer_block: u64,
    ) {
        self.broadcast_message(&BitmapMessage::BitmapTransfer {
            blockheight: blockheight.to_string(),
            new_owner: new_owner.to_string(),
            transfer_block,
        });
    }

    // BNS subtopic via command naming convention: bitmap_bns_<command>
    // This wrapper corresponds to: bitmap_bns_claim
    pub fn send_bitmap_bns_claim(&mut self, entry: &super::database::BnsEntry) {
        self.broadcast_message(&BitmapMessage::BnsInscription { entry: entry.clone() });
    }
}
