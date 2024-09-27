use anyhow::{self, bail, Context, Ok};
use serde::{Deserialize, Serialize};
use serde_json::{self};
use std::io::{stdin, stdout, StdoutLock, Write};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct Body {
    #[serde(flatten)]
    payload: Payload,
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    #[serde(rename = "in_reply_to")]
    to: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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

struct EchoNode {
    id: usize,
}

impl EchoNode {
    pub fn step(&mut self, input: Message, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Init { .. } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        to: input.body.id,
                        payload: Payload::InitOk,
                        id: Some(self.id),
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to Init")?;
                output.write_all(b"\n").context("write trailing newline")?;

                self.id += 1;
            }
            Payload::InitOk => bail!("Received init_ok message"),
            Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        to: input.body.id,
                        payload: Payload::EchoOk { echo },
                        id: Some(self.id),
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to Echo")?;
                output.write_all(b"\n").context("write trailing newline")?;

                self.id += 1;
            }
            Payload::EchoOk { echo: _ } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let _stdin = stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(_stdin).into_iter::<Message>();

    let mut _stdout = stdout().lock();
    let mut node = EchoNode { id: 0 };

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialized")?;

        node.step(input, &mut _stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_out() -> Result<(), anyhow::Error> {
        // let mut _stdout = AddNewlineWriter(stdout().lock());
        let mut _stdout = stdout().lock();
        let mut node = EchoNode { id: 0 };

        let input = Message {
            src: String::from("n1"),
            dst: String::from("n2"),
            body: Body {
                payload: Payload::Echo {
                    echo: String::from("Hi"),
                },
                id: Some(1),
                to: Some(2),
            },
        };

        node.step(input, &mut _stdout)
            .context("Node step function failed")?;

        Ok(())
    }
}
