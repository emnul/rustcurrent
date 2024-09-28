use anyhow::{self, Context, Ok};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{self};
use std::io::{stdin, stdout, StdoutLock};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Body<Payload> {
    #[serde(flatten)]
    pub payload: Payload,
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    #[serde(rename = "in_reply_to")]
    pub to: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<Payload> {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, Payload>(mut state: S) -> anyhow::Result<()>
where
    S: Node<Payload>,
    Payload: DeserializeOwned,
{
    let _stdin = stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(_stdin).into_iter::<Message<Payload>>();

    let mut _stdout = stdout().lock();

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialized")?;

        state
            .step(input, &mut _stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}
