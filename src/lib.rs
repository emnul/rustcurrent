use anyhow::{self, Context, Ok};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{self};
use std::{
    io::{stdin, stdout, BufRead, StdoutLock, Write},
    sync::mpsc::Sender,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

// Payload agnostic impl
impl<Payload> Message<Payload> {
    pub fn into_reply(self, id: Option<&mut usize>) -> Self {
        Self {
            src: self.dst,
            dst: self.src,
            body: Body {
                to: self.body.id,
                payload: self.body.payload,
                id: id.map(|id| {
                    let mid = *id;
                    *id += 1;
                    mid
                }),
            },
        }
    }

    pub fn send(&self, output: &mut impl Write) -> anyhow::Result<()>
    where
        Payload: Serialize,
    {
        serde_json::to_writer(&mut *output, self).context("serialize response message")?;
        output.write_all(b"\n").context("write trailing newline")?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Event<Payload, InjectedPayload = ()> {
    Message(Message<Payload>),
    Injected(InjectedPayload),
    EOF,
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
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<S, Payload, InjectedPayload = ()> {
    fn from_init(
        state: S,
        init: Init,
        inject: Sender<Event<Payload, InjectedPayload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn step(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()>;
}

/// Main loop now handles input message by reading the first message from stdin which is guaranteed
/// to be an Init Message
pub fn main_loop<S, N, P, IP>(init_state: S) -> anyhow::Result<()>
where
    P: DeserializeOwned + Send + 'static,
    N: Node<S, P, IP>,
    // Channels require types to be Send
    IP: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();

    let stdin = stdin().lock();
    let mut stdin = stdin.lines();
    let mut _stdout = stdout().lock();

    // We use stdin lines here bc we know that the format is newline separated
    // A StreamDeserializer checks whether it can deserialize at the
    // end of newlines which is more overhead that we can easily avoid
    let init_msg: Message<InitPayload> = serde_json::from_str(
        &stdin
            .next()
            .expect("no init message received")
            .context("failed to read init message from stdin")?,
    )
    .context("init message could not be deserialized")?;

    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("first message should be init");
    };
    // providing tx handle to node allows it to inject it's own messges
    let mut node: N =
        Node::from_init(init_state, init, tx.clone()).context("node initialization failed")?;

    let reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            to: init_msg.body.id,
            payload: InitPayload::InitOk,
            // We're reserving 0 for the init_ok message
            // Starting nodes with id = 1
            id: Some(0),
        },
    };
    serde_json::to_writer(&mut _stdout, &reply).context("serialize response to Init")?;
    _stdout.write_all(b"\n").context("write trailing newline")?;

    // dropping the stdinlock doesn't mean that any buffer data from stdin will also be dropped
    drop(stdin);
    let jh = std::thread::spawn(move || {
        let stdin = std::io::stdin().lock();

        for line in stdin.lines() {
            let line = line.context("Maelstrom input from STDIN could not be read")?;
            let input: Message<P> = serde_json::from_str(&line)
                .context("Maelstrom input from STDIN could not be deserialized")?;
            if let Err(_) = tx.send(Event::Message(input)) {
                return Ok(());
            }
        }
        // Add a way for the node to learn it should exit
        let _ = tx.send(Event::EOF);
        Ok(())
    });

    for input in rx {
        node.step(input, &mut _stdout)
            .context("Node step function failed")?;
    }

    // wait for thread once channel has closed
    jh.join()
        .expect("stdin thread panicked")
        .context("stdin thread returned an error")?;

    Ok(())
}
