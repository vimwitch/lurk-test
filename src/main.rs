use std::{sync::Arc};
use halo2curves::{bn256::{Bn256, Fr as Bn}, ff::derive::rand_core::le};
use lurk::{
    dual_channel::dummy_terminal, field::LurkField, lang::Lang, lem::{eval::{evaluate, evaluate_simple, evaluate_with_env, make_cprocs_funcs_from_lang, make_eval_step_from_config, EvalConfig}, pointers::Ptr, store::Store, tag::Tag}, proof::{nova::NovaProver, Prover, RecursiveSNARKTrait}, public_parameters::{instance::{Instance, Kind}, public_params}, state::user_sym, tag::ContTag
};

const REDUCTION_COUNT: usize = 2;
fn main() {
    let program = include_str!("test.lurk");
    println!("{:?}", program);

    let store = &Store::<Bn>::default();
    let call = store.read_with_default_state(program).unwrap();
    println!("Starting...");

    let lang = Lang::<Bn>::new();
    let lang_rc = Arc::new(lang.clone());
    let config = EvalConfig::new_ivc(&lang);

    // first we're going to evaluate the program to see if it's valid
    let eval_step = make_eval_step_from_config(&config);
    let cprocs = make_cprocs_funcs_from_lang(&lang.clone());

    // first check if the program is validish?
    let frames = evaluate(Some((&eval_step, &cprocs[..], &lang_rc)), call, &store, 10000, &dummy_terminal::<Ptr>());
    let output = frames.unwrap().last().expect("no frames").output.clone();
    println!("{:?}", output[2]);
    match output[2].tag() {
        Tag::Cont(ContTag::Terminal) => {}
        Tag::Cont(ContTag::Error) => {
            println!("program has error");
        }
        _ => println!("needs more iterations")
    }
    
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
    let proof_c = proof.compress(&pp).unwrap();
    println!("Compressed proof");
    assert!(proof_c.verify(&pp, &z0, &zi).unwrap());
    println!("Verified compressed proof");

    let buf = bincode::serialize(&proof_c).unwrap();
    println!("proof size: {:?} bytes", buf.len());
}
