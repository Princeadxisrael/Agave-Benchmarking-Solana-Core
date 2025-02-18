#![allow(clippy::arithmetic_side_effects)]


use {
crossbeam_channel::{unbounded, bounded, Receiver},
log::*,
rand::{thread_rng, Rng},
rayon::prelude::*,
solana_client::connection_cache::ConnectionCache,
solana_core::{
    banking_stage::{
        committer::Committer,
        consumer::Consumer,
        leader_slot_metrics::LeaderSlotMetricsTracker,
        qos_service::QosService,
        unprocessed_packet_batches::*,
        unprocessed_transaction_storage::{ThreadType, UnprocessedTransactionStorage},
        BankingStage, BankingStageStats,
    },
    banking_trace::{BankingPacketBatch, BankingTracer},
},
solana_entry::entry::{next_hash, Entry},
solana_gossip::cluster_info::{ClusterInfo, Node},
solana_ledger::{
    blockstore::Blockstore,
    blockstore_processor::process_entries_for_tests,
    genesis_utils::{create_genesis_config, GenesisConfigInfo},
    get_tmp_ledger_path_auto_delete,
},
solana_perf::{
    packet::{to_packet_batches, Packet},
    test_tx::test_tx,
},
solana_poh::poh_recorder::{create_test_recorder, WorkingBankEntry},
solana_runtime::{
    bank::Bank, bank_forks::BankForks, prioritization_fee_cache::PrioritizationFeeCache,
},
solana_sdk::{
    genesis_config::GenesisConfig,
    hash::Hash,
    message::Message,
    pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_transaction,
    timing::timestamp,
    transaction::{Transaction, VersionedTransaction},
},
solana_streamer::socket::SocketAddrSpace,
std::{
    iter::repeat_with,
    sync::{atomic::Ordering, Arc},
    time::{Duration, Instant},
},
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};


//helper function to verify all expected transactions were processed
//uses a 1‑second timeout for each receive attempt and breaks after 60 seconds if the expected count is not met.
fn check_txs(receiver: &Arc<Receiver<WorkingBankEntry>>, ref_tx_count: usize) {
    let mut total = 0;
    let now = Instant::now();
    loop {
        if let Ok((_bank, (entry, _tick_height))) = receiver.recv_timeout(Duration::new(1, 0)) {
            total += entry.transactions.len();
        }
        if total >= ref_tx_count {
            break;
        }
        if now.elapsed().as_secs() > 60 {
            break;
        }
    }
    assert_eq!(total, ref_tx_count);
}

fn make_accounts_txs(txes: usize, mint_keypair: &Keypair, hash: Hash) -> Vec<Transaction> {
    let to_pubkey = pubkey::new_rand();
    let dummy = system_transaction::transfer(mint_keypair, &to_pubkey, 1, hash);
    (0..txes)
        .into_par_iter()
        .map(|_| {
            let mut new = dummy.clone();
            let sig: [u8; 64] = std::array::from_fn(|_| thread_rng().gen::<u8>());
            new.message.account_keys[0] = pubkey::new_rand();
            new.message.account_keys[1] = pubkey::new_rand();
            new.signatures = vec![Signature::from(sig)];
            new
        })
        .collect()
}

fn make_programs_txs(txes: usize, hash: Hash) -> Vec<Transaction> {
    let progs = 4;
    (0..txes)
        .map(|_| {
            let from_key = Keypair::new();
            let instructions: Vec<_> = repeat_with(|| {
                let to_key = pubkey::new_rand();
                system_instruction::transfer(&from_key.pubkey(), &to_key, 1)
            })
            .take(progs)
            .collect();
            let message = Message::new(&instructions, Some(&from_key.pubkey()));
            Transaction::new(&[&from_key], message, hash)
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
enum QueueType {
    Unbounded,
    Bounded(usize),
}

fn bench_transaction_scheduler(c: &mut Criterion) {
    let num_threads = BankingStage::num_threads() as usize;
    const PACKETS_PER_BATCH: usize = 192;
    let txes = 192 * num_threads * 8;
    let mint_total = 1_000_000_000_000;
    let GenesisConfigInfo { mut genesis_config, mint_keypair, .. } = create_genesis_config(mint_total);

    genesis_config.ticks_per_slot = 10_000;
    
    let mut bank = Bank::new_for_benches(&genesis_config);
    bank.ns_per_slot = u128::MAX;
    let bank_forks = BankForks::new_rw_arc(bank);
    let bank = bank_forks.read().unwrap().get(0).unwrap();
    bank.write_cost_tracker().unwrap().set_limits(u64::MAX, u64::MAX, u64::MAX);

    let transactions = make_accounts_txs(txes, &mint_keypair, genesis_config.hash());

    // create a queue
    let create_channel = |queue_type: QueueType| -> (crossbeam_channel::Sender<_>, crossbeam_channel::Receiver<_>) {
        match queue_type {
            QueueType::Unbounded => unbounded(),
            QueueType::Bounded(size) => bounded(size),
        }
    };
    
    
    let mut group = c.benchmark_group("transaction_scheduler");
    
    //create queue type and bounded values
    //unbounded queues maximize PS but unrealistic
    //Bounded queues (1024) simulate congestion and reveal queueing delays
    //Bounded queues (1024) balances throughput and delay
    for queue_type in &[QueueType::Unbounded, QueueType::Bounded(1024), QueueType::Bounded(8192)] {
        let (non_vote_sender, non_vote_receiver) = create_channel(*queue_type);

        group.bench_function(&format!("{:?}", queue_type), |b| {
            b.iter(|| {
                let now = Instant::now();
                //convert transaction to packetBatch objects
                let packet_batches = to_packet_batches(&transactions, PACKETS_PER_BATCH);
                //transmit packetized transcations to the banking stage
                for batch in packet_batches {
                
                //measure throughput at the transaction queue level
                non_vote_sender.send(BankingPacketBatch::new((vec![batch], None))).unwrap();
                }
                let elapsed = now.elapsed();
                println!("Queue {:?}: Time: {:?}", queue_type, elapsed);
            });
        });
    }
    group.finish();
}


criterion_group!(benches, bench_transaction_scheduler);
criterion_main!(benches);