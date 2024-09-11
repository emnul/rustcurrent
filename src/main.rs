use serde::{Deserialize, Serialize};

// https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md#messages
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Body<B> {
    #[serde(rename = "type")]
    ty: String,
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,

    #[serde(flatten)]
    rest: B,
}

fn main() {
    println!("Hello, world!");
}
