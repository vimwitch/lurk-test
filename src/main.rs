use std::{sync::Arc};
use halo2curves::bn256::{Bn256, Fr as Bn};
use lurk::{
    dual_channel::dummy_terminal, field::LurkField, lang::Lang, lem::{pointers::Ptr, store::Store}, proof::{nova::NovaProver, Prover, RecursiveSNARKTrait}, public_parameters::{instance::{Instance, Kind}, public_params}, state::user_sym
};

fn build_program<F: LurkField>(store: &Store<F>) -> Ptr {
    let program = "(+ 2 2)";
    store.read_with_default_state(program).unwrap()
}

const REDUCTION_COUNT: usize = 2;
fn main() {
    println!("Starting...");
    let store = &Store::<Bn>::default();
    let lang = Lang::<Bn>::new();
    let lang_rc = Arc::new(lang.clone());
    let call = build_program(&store);
    
    let nova_prover = NovaProver::new(REDUCTION_COUNT, lang_rc.clone());

    let instance = Instance::new(REDUCTION_COUNT, lang_rc, true, Kind::NovaPublicParams);
    let pp = public_params(&instance).unwrap();
    let (proof, z0, zi, _num_steps) = nova_prover.evaluate_and_prove(
        &pp,
        call,
        store.intern_empty_env(),
        store,
        10000,
       &dummy_terminal() 
    ).unwrap();
    println!("Built proof");
    assert!(proof.verify(&pp, &z0, &zi).unwrap());
    println!("Verified proof");
}
