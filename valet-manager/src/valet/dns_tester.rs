#![allow(dead_code)]

pub async fn run_dig(
    host: &str,
    tx: tokio::sync::mpsc::Sender<crate::creator::output_streamer::OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    let args = ["@127.0.0.1", host];
    let status = crate::creator::output_streamer::stream_command(
        "dig", &args, None, tx, cancel_rx,
    )
    .await?;
    Ok(status.success())
}

pub async fn change_tld(
    tld: &str,
    tx: tokio::sync::mpsc::Sender<crate::creator::output_streamer::OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    let args = ["domain", tld];
    let status = crate::creator::output_streamer::stream_command(
        "valet", &args, None, tx, cancel_rx,
    )
    .await?;
    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_dig_signature_compiles() {
        // Compile-time sanity: the function pointer resolves to expected shape.
        let _f: fn(
            &'static str,
            tokio::sync::mpsc::Sender<crate::creator::output_streamer::OutputLine>,
            tokio::sync::watch::Receiver<bool>,
        ) -> _ = |host, tx, rx| run_dig(host, tx, rx);
    }

    #[test]
    fn change_tld_signature_compiles() {
        let _f: fn(
            &'static str,
            tokio::sync::mpsc::Sender<crate::creator::output_streamer::OutputLine>,
            tokio::sync::watch::Receiver<bool>,
        ) -> _ = |tld, tx, rx| change_tld(tld, tx, rx);
    }
}
