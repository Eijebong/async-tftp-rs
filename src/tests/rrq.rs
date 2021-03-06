#![cfg(feature = "external-client-tests")]

use futures::channel::oneshot;
use smol::Task;

use super::external_client::*;
use super::handlers::*;
use crate::server::TftpServerBuilder;

fn transfer(file_size: usize, block_size: Option<u16>) {
    smol::run(async {
        let (md5_tx, md5_rx) = oneshot::channel();
        let handler = RandomHandler::new(file_size, md5_tx);

        // bind
        let tftpd = TftpServerBuilder::with_handler(handler)
            .bind("127.0.0.1:0".parse().unwrap())
            .build()
            .await
            .unwrap();
        let addr = tftpd.listen_addr().unwrap();

        // start client
        let tftp_recv = Task::blocking(async move {
            external_tftp_recv("test", addr, block_size)
        });

        // start server
        Task::spawn(async move {
            tftpd.serve().await.unwrap();
        })
        .detach();

        // check md5
        let client_md5 = tftp_recv.await.expect("failed to run tftp client");
        let server_md5 = md5_rx.await.expect("failed to receive server md5");
        assert_eq!(client_md5, server_md5);
    });
}

#[test]
fn transfer_0_bytes() {
    transfer(0, None);
    transfer(0, Some(1024));
}

#[test]
fn transfer_less_than_block() {
    transfer(1, None);
    transfer(123, None);
    transfer(511, None);
    transfer(1023, Some(1024));
}

#[test]
fn transfer_block() {
    transfer(512, None);
    transfer(1024, Some(1024));
}

#[test]
fn transfer_more_than_block() {
    transfer(512 + 1, None);
    transfer(512 + 123, None);
    transfer(512 + 511, None);
    transfer(1024 + 1, Some(1024));
    transfer(1024 + 123, Some(1024));
    transfer(1024 + 1023, Some(1024));
}

#[test]
fn transfer_1mb() {
    transfer(1024 * 1024, None);
    transfer(1024 * 1024, Some(1024));
}

#[test]
#[ignore]
fn transfer_almost_32mb() {
    transfer(32 * 1024 * 1024 - 1, None);
}

#[test]
#[ignore]
fn transfer_32mb() {
    transfer(32 * 1024 * 1024, None);
}

#[test]
#[ignore]
fn transfer_more_than_32mb() {
    transfer(33 * 1024 * 1024 + 123, None);
}

#[test]
#[ignore]
fn transfer_more_than_64mb() {
    transfer(65 * 1024 * 1024 + 123, None);
}
