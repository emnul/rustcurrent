use std::io::StdoutLock;

use anyhow::Context;
use serde::{Deserialize, Serialize};

// https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md#messages
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

// serde flatten field attr: Flatten the contents of this field into the container it is defined in.
//
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

// serde tag container attr: On an enum: Use the internally tagged enum representation, with the given tag. See enum representations for details on this representation.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

// state machine
struct EchoNode {
    id: usize,
}

impl EchoNode {
    // state machine step function
    pub fn step(
        &mut self,
        input: Message,
        // state machine may want to send messages while it's executing as well
        output: &mut serde_json::Serializer<StdoutLock>,
    ) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                reply
                    .serialize(output)
                    .context("Serialize reponse to echo")?;
                self.id += 1;
            }
            Payload::EchoOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let stdout = std::io::stdout().lock();
    let mut output = serde_json::Serializer::new(stdout);

    let mut state = EchoNode { id: 0 };

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialized")?;
        state
            .step(input, &mut output)
            .context("Node step function failed")?;
    }

    Ok(())
}
