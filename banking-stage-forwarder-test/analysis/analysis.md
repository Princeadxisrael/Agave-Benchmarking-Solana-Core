# Analysis on Test done on Forwarder



#![allow(clippy::arithmetic_side_effects)]
// #![feature(test)]


use solana_ledger::blockstore_options::BlockstoreOptions;

use {
        itertools::Itertools,
        solana_client::connection_cache::ConnectionCache,
        solana_core::{
            banking_stage::{
                forwarder::Forwarder,
                leader_slot_metrics::LeaderSlotMetricsTracker,
                unprocessed_packet_batches::{DeserializedPacket, UnprocessedPacketBatches},
                unprocessed_transaction_storage::{ThreadType, UnprocessedTransactionStorage},
                BankingStageStats,
            },
            tracer_packet_stats::TracerPacketStats,
        },
     
        solana_gossip::cluster_info::{ClusterInfo, Node},
        solana_ledger::{
            blockstore::Blockstore,
            genesis_utils::{create_genesis_config_with_leader, GenesisConfigInfo},
        },
        solana_perf::{data_budget::DataBudget, packet::Packet},
        solana_poh::{poh_recorder::create_test_recorder, poh_service::PohService},
        solana_runtime::{bank::Bank, genesis_utils::bootstrap_validator_stake_lamports},
        solana_sdk::{poh_config::PohConfig, signature::Keypair, signer::Signer, system_transaction},
        solana_streamer::socket::SocketAddrSpace,
        std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        tempfile::TempDir,
        // test::Bencher,
    };



use {
    criterion::{criterion_group, criterion_main, Criterion},
    
    std::time::Instant,
    
};

struct BenchSetup {
    exit: Arc<AtomicBool>,
    poh_service: PohService,
    forwarder: Forwarder<Arc<ClusterInfo>>,
    unprocessed_packet_batches: UnprocessedTransactionStorage,
    tracker: LeaderSlotMetricsTracker,
    stats: BankingStageStats,
    tracer_stats: TracerPacketStats,
}
//initialize a minimal validator environment
fn setup(num_packets: usize, contentious_transaction: bool) -> BenchSetup {
    let validator_keypair = Arc::new(Keypair::new());
    let genesis_config_info = create_genesis_config_with_leader(
        10_000,
        &validator_keypair.pubkey(),
        bootstrap_validator_stake_lamports(),
    );
    let GenesisConfigInfo { genesis_config, .. } = &genesis_config_info;
    
    //simulate a Solana validator and no wall-clock throtling to prevent artificial delay in execution
    let (bank, bank_forks) = Bank::new_no_wallclock_throttle_for_tests(genesis_config);
    let mut options = BlockstoreOptions::default();
    options.enforce_ulimit_nofile=false;
    let ledger_path = TempDir::new().unwrap();
    let blockstore = Arc::new(
        Blockstore::open_with_options(ledger_path.as_ref(),options)
            .expect("Expected to be able to open database ledger"),
    );
    let poh_config = PohConfig {
        // limit tick count to avoid clearing working_bank at
        // PohRecord then PohRecorderError(MaxHeightReached) at BankingStage
        target_tick_count: Some(bank.max_tick_height().saturating_sub(1)),
        ..PohConfig::default()
    };

    let (exit, poh_recorder, poh_service, _entry_receiver) =
        create_test_recorder(bank, blockstore, Some(poh_config), None);

    let local_node = Node::new_localhost_with_pubkey(&validator_keypair.pubkey());
    let cluster_info = ClusterInfo::new(
        local_node.info.clone(),
        validator_keypair,
        SocketAddrSpace::Unspecified,
    );
    let cluster_info = Arc::new(cluster_info);
    let min_balance = genesis_config.rent.minimum_balance(0);
    let hash = genesis_config.hash();

    // packets are deserialized upon receiving, failed packets will not be
    // forwarded; Therefore need to create real packets here.
    let keypair = Keypair::new();
    let packets = (0..num_packets)
        .map(|_| {
            let mut transaction =
                system_transaction::transfer(&keypair, &Keypair::new().pubkey(), min_balance, hash);
            if !contentious_transaction {
                transaction.message.account_keys[0] = solana_sdk::pubkey::Pubkey::new_unique();
            }
            let mut packet = Packet::from_data(None, transaction).unwrap();
            packet.meta_mut().set_tracer(true);
            packet.meta_mut().set_from_staked_node(true);
            DeserializedPacket::new(packet).unwrap()
        })
        .collect_vec();

    let unprocessed_packet_batches = UnprocessedTransactionStorage::new_transaction_storage(
        UnprocessedPacketBatches::from_iter(packets, num_packets),
        ThreadType::Transactions,
    );

    let connection_cache = ConnectionCache::new("connection_cache_test");
    // use a restrictive data budget to bench everything except actual sending data over
    // connection.
    let data_budget = DataBudget::restricted();
    let forwarder = Forwarder::new(
        poh_recorder,
        bank_forks,
        cluster_info,
        Arc::new(connection_cache),
        Arc::new(data_budget),
    );

    BenchSetup {
        exit,
        poh_service,
        forwarder,
        unprocessed_packet_batches,
        tracker: LeaderSlotMetricsTracker::new(0),
        stats: BankingStageStats::default(),
        tracer_stats: TracerPacketStats::new(0),
    }
}

fn bench_forwarder_throughput_and_latency(c: &mut Criterion) {
    let num_packets = 10240; // Number of packets to process per batch.
    let BenchSetup {
        exit,
        poh_service,
        mut forwarder,
        mut unprocessed_packet_batches,
        mut tracker,
        stats,
        mut tracer_stats,
    } = setup(num_packets, false);

    c.bench_function("forwarder_throughput_and_latency", |b| {
        b.iter(|| {
            // Measure latency
            let start_time = Instant::now();
            forwarder.handle_forwarding(
                &mut unprocessed_packet_batches,
                true, // Hold packets for reuse
                &mut tracker,
                &stats,
                &mut tracer_stats,
            );
            let elapsed_time = start_time.elapsed();

            // Reset the forwarded flag for reuse
            if let UnprocessedTransactionStorage::LocalTransactionStorage(unprocessed_packets) =
                &mut unprocessed_packet_batches
            {
                for deserialized_packet in unprocessed_packets.iter_mut() {
                    deserialized_packet.forwarded = false;
                }
            }

            // Calculate throughput
            let packets_processed = unprocessed_packet_batches.len();
            let throughput_packets_per_second =
                packets_processed as f64 / elapsed_time.as_secs_f64();

            // Log results (optional)
            println!(
                "Throughput: {:.2} packets/second, Latency: {:.2} µs",
                throughput_packets_per_second,
                elapsed_time.as_micros() as f64 / packets_processed as f64
            );
        });
    });

    // Cleanup
    exit.store(true, Ordering::Relaxed);
    poh_service.join().unwrap();
}

// Criterion test groups
criterion_group!(benches, bench_forwarder_throughput_and_latency);
criterion_main!(benches);
