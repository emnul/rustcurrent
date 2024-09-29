use anyhow::{self, bail, Context, Ok};
use rustcurrent::*;
use serde::{Deserialize, Serialize};
use serde_json::{self};
use std::io::{StdoutLock, Write};
use ulid::Ulid;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

struct UniqueNode {
    id: usize,
}

impl Node<Payload> for UniqueNode {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
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
            Payload::Generate => {
                let guid = Ulid::new().to_string();
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        to: input.body.id,
                        payload: Payload::GenerateOk { guid },
                        id: Some(self.id),
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to generate")?;
                output.write_all(b"\n").context("write trailing newline")?;

                self.id += 1;
            }
            Payload::GenerateOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop(UniqueNode { id: 0 })
}
