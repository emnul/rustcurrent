use std::io::{StdoutLock, Write};

use anyhow::{bail, Context};
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
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
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
        // We changed the function signature here to StdoutLock bc the Maelstrom protocol requires that
        // we write messages as JSON objects to Stdout *separated by newlines*
        // We can't do this with serde_json::ser::PrettyFormatter AFAIK
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Init { .. } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &reply).context("Serialize reponse to init")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
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
                // need to perform a reborrow here so we can use stdout again after
                // writer is done
                serde_json::to_writer(&mut *output, &reply).context("Serialize reponse to init")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            Payload::InitOk { .. } => bail!("received init ok message"),
            Payload::EchoOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    // our Desrializer can be turned into an iterator. This is desirable because
    // we know there will be multiple things we're going to deserialize
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = std::io::stdout().lock();

    let mut state = EchoNode { id: 0 };

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialized")?;
        state
            .step(input, &mut stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}
