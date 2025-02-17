use std::io::{StdoutLock, Write};
use std::process::Output;
use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo {echo: String},
    EchoOk {echo: String},
    Init {node_id: String, node_ids: Vec<String>},
    InitOk,
}

struct EchoNode {
    id: usize,
}

impl EchoNode {
    fn step(
        &mut self,
        input: Message,
        output: &mut StdoutLock
    ) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Init { .. } => {
                let reply = Message {
                    src : input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk
                    }
                };
                serde_json::to_writer(&mut *output, &reply).context("serialize response to init")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            Payload::Echo {echo} => {
                let reply = Message {
                    src : input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk {echo}
                    }
                };
                serde_json::to_writer(&mut *output, &reply).context("serialize response to echo")?;
                output.write_all(b"\n").context("write trailing newline")?;
                //reply.serialize(output).context("serialize response to echo")?;
                self.id += 1;
            }
            Payload::EchoOk { ..} => bail!("received init_ok message"),
            Payload::InitOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = std::io::stdout().lock();
    //let mut output = serde_json::Serializer::pretty(stdout);

    let mut state = EchoNode {
        id: 0
    };

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialize")?;
        state.step(input, &mut stdout).context("Node step function failed")?;
    }

    Ok(())
}
