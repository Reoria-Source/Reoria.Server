#![allow(dead_code)]
#![recursion_limit = "256"]
#![feature(test, let_chains)]
extern crate test;
#[macro_use]
extern crate derivative;

mod containers;
mod gameloop;
mod gametypes;
mod items;
mod maps;
mod npcs;
mod players;
mod socket;
mod sql;
mod tasks;
mod time_ext;

#[allow(unused_imports)]
use gameloop::*;
use gametypes::*;
//use npcs::*;
//use player::Player;
//use serde::{Deserialize, Serialize};
//use socket::*;
//use sql::*;
use containers::Storage;
//use test::Bencher;
//use bytey::ByteBuffer;
//use bytey::{ByteBufferRead, ByteBufferWrite};
//use time_ext::{MyDuration, MyInstant};
use hecs::World;

fn read_line() -> String {
    let mut rv = String::new();
    std::io::stdin().read_line(&mut rv).unwrap();
    rv.replace("\r\n", "").replace('\n', "")
}

fn main() {
    let storage = match Storage::new() {
        Some(n) => n,
        None => return,
    };
    let router = PacketRouter::init();
    let mut world = World::new();

    game_loop(&mut world, &storage, &router);
    println!("done. Press enter to exit program.");

    let _ret = read_line();
}
