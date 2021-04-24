//! Code to generate device definitions

use std::{borrow::BorrowMut, convert::TryInto, path::PathBuf};

use anyhow::Result;
use clap::{self as clap, Clap};
use codegen::{m2x4hd, DeviceSpec};
use futures::{Stream, StreamExt};
use minidsp::{
    commands::Commands,
    utils::{decoder, recorder},
};
use tokio::{fs::File, io::AsyncReadExt};
use tokio_util::{
    codec::{Decoder, LinesCodec},
    io::StreamReader,
};

mod codegen;

#[derive(Clap, Debug)]
#[clap(version=env!("CARGO_PKG_VERSION"), author=env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    /// Pretty-print protocol decodes
    Decode { input: PathBuf },

    /// Dumps the bulk-loaded parameter data into a file
    DumpBulk {
        input: PathBuf,
        output: PathBuf,
        #[clap(long)]
        skip: Option<usize>,
    },

    Codegen {
        /// The directory prefix where generated files should be written
        /// This should map to minidsp_protocol/src/device/
        output: PathBuf,
    },
}

#[tokio::main]
pub async fn main() -> Result<()> {
    env_logger::init();
    let opts: Opts = Opts::parse();

    match opts.cmd {
        SubCommand::Decode { input } => {
            let file = File::open(input).await?;
            let framed = LinesCodec::new().framed(file);
            let messages =
                framed.filter_map(|x| async { recorder::Message::from_string(x.ok()?.as_str()) });
            decode(messages).await?;
        }
        SubCommand::DumpBulk {
            input,
            output,
            skip,
        } => {
            let file = File::open(input).await?;
            let framed = LinesCodec::new().framed(file);
            let messages =
                framed.filter_map(|x| async { recorder::Message::from_string(x.ok()?.as_str()) });
            dump(output, skip, messages).await?;
        }
        SubCommand::Codegen { output } => {
            codegen_main(output)?;
        }
    }

    Ok(())
}

async fn dump(
    output: PathBuf,
    skip: Option<usize>,
    framed: impl Stream<Item = recorder::Message>,
) -> Result<()> {
    // Only keep bulk load commands
    let f = framed
        .filter_map(recorder::decode_sent_commands)
        .filter_map(|x| async {
            match x {
                Commands::BulkLoad { payload } => Some(Ok::<_, std::io::Error>(payload.0)),
                _ => None,
            }
        });

    let mut reader = Box::pin(StreamReader::new(f));
    let mut output = File::create(output).await?;

    if let Some(skip) = skip {
        tokio::io::copy(
            &mut reader.borrow_mut().take(skip.try_into().unwrap()),
            &mut tokio::io::sink(),
        )
        .await?;
    }

    tokio::io::copy(&mut reader, &mut output).await?;

    Ok(())
}

async fn decode(framed: impl Stream<Item = recorder::Message>) -> Result<()> {
    let mut decoder = {
        use termcolor::{ColorChoice, StandardStream};
        let writer = StandardStream::stdout(ColorChoice::Always);
        decoder::Decoder::new(Box::new(writer), true)
    };

    let mut n_recv: i32 = 0;
    let mut n_sent: i32 = 0;
    let mut framed = Box::pin(framed);

    while let Some(msg) = framed.next().await {
        match msg {
            recorder::Message::Sent(data) => {
                n_sent += 1;
                print!("{}:", n_sent);
                decoder.feed_sent(&data);
            }
            recorder::Message::Received(data) => {
                n_recv += 1;
                print!("{}:", n_recv);
                decoder.feed_recv(&data);
            }
        }
    }

    Ok(())
}

fn gen_m2x4hd() -> String {
    let s = m2x4hd::Spec::new()
        .generate_static_config("DEVICE")
        .to_string();

    "//\n// This file is generated by `minidsp-devtools codegen`. DO NOT EDIT.\n//\n".to_string()
        + &s
}

fn codegen_main(output: PathBuf) -> Result<()> {
    std::fs::write(output.join("m2x4hd.rs"), gen_m2x4hd())?;
    Ok(())
}
