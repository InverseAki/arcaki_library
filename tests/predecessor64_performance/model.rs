include!("common.rs");
include!("model_checks.rs");

#[test] fn baseline_model() { verify::<baseline::Predecessor64>(); }
#[test] fn update_model() { verify::<update_only::Predecessor64>(); }
#[test] fn incremental_model() { verify::<candidates::Incremental>(); }
#[test] fn leaf_model() { verify::<candidates::LeafFirst>(); }
#[test] fn flat_model() { verify::<flat::Flat>(); }
#[test] fn shifted_model() { verify::<candidates::Shifted>(); }
#[test] fn flat_shifted_model() { verify::<flat::FlatShifted>(); }
#[test] fn current_model() { verify::<current::Predecessor64>(); }
