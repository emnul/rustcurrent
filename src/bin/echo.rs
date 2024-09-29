use anyhow::{self, Context, Ok};
use rustcurrent::*;
use serde::{Deserialize, Serialize};
use serde_json::{self};
use std::io::{StdoutLock, Write};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {
    id: usize,
}

impl Node<(), Payload> for EchoNode {
    fn from_init(_state: (), _init: rustcurrent::Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(EchoNode { id: 1 })
    }
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
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
    main_loop::<_, EchoNode, _>(())
}
