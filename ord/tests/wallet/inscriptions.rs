use {
  super::*,
  ord::subcommand::wallet::{inscriptions, receive, send},
};

#[test]
fn inscriptions() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, reveal) = inscribe(&core, &ord);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{reveal}:0:0").parse().unwrap());
  assert_eq!(
    output[0].explorer,
    format!("https://ordinals.com/inscription/{inscription}")
  );

  let addresses = CommandBuilder::new("wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<receive::Output>()
    .addresses;

  let destination = addresses.first().unwrap();

  let txid = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 {} {inscription}",
    destination.clone().assume_checked()
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run_and_deserialize_output::<send::Output>()
  .txid;

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{txid}:0:0").parse().unwrap());
}

#[test]
fn inscriptions_includes_locked_utxos() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, reveal) = inscribe(&core, &ord);

  core.mine_blocks(1);

  core.lock(OutPoint {
    txid: reveal,
    vout: 0,
  });

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{reveal}:0:0").parse().unwrap());
}

#[test]
fn inscriptions_with_postage() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, _) = inscribe(&core, &ord);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output[0].postage, 10000);

  let addresses = CommandBuilder::new("wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<receive::Output>()
    .addresses;

  let destination = addresses.first().unwrap();

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 {} {inscription}",
    destination.clone().assume_checked()
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run_and_extract_stdout();

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output[0].postage, 9889);
}
