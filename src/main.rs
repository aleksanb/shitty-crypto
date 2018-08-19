#[macro_use]
extern crate structopt;

use std::fs::File;
use std::io::{self, prelude::*};
use std::ops::BitXor;
use std::ops::Shl;
use structopt::StructOpt;

type Rule = [u8; 8];

macro_rules! ca_rule {
    ($num:expr) => {{
        let rule: Rule = [
            $num.rotate_right(0) & 1,
            $num.rotate_right(1) & 1,
            $num.rotate_right(2) & 1,
            $num.rotate_right(3) & 1,
            $num.rotate_right(4) & 1,
            $num.rotate_right(5) & 1,
            $num.rotate_right(6) & 1,
            $num.rotate_right(7) & 1,
        ];
        rule
    }};
}

fn transition(previous_state: u32, rule: Rule) -> u32 {
    let previous_state = previous_state.rotate_left(1);
    let mut next_state = 0u32;
    for column in 0..32 {
        let shifted_state = previous_state.rotate_right(column);
        let rule_index = shifted_state & 0b111;
        let next_value = u32::from(rule[rule_index as usize]);
        next_state |= next_value.shl(column);
    }
    next_state
}

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(long = "encrypt")]
    encrypt: Option<String>,

    #[structopt(long = "decrypt")]
    decrypt: Option<String>,

    #[structopt(long = "ca_rule", default_value = "110")]
    cellular_automata: u8,

    #[structopt(long = "iterations", default_value = "1024")]
    iterations: usize,
}

fn u32_to_4_u8(source: u32) -> [u8; 4] {
    let first = (source & 0b1111_1111) as u8;
    let second = ((source >> 8) & 0b1111_1111) as u8;
    let third = ((source >> 16) & 0b1111_1111) as u8;
    let fourth = ((source >> 24) & 0b1111_1111) as u8;
    [first, second, third, fourth]
}

fn create_pad(bytes_needed: usize, cellular_automata: u8, iterations: usize) -> Vec<u8> {
    let ca_rule = ca_rule!(cellular_automata);

    let mut bytes_for_encryption = vec![];
    let mut current_state = 1u32;

    for _ in 0..iterations {
        current_state = transition(current_state, ca_rule);
    }

    while bytes_for_encryption.len() < bytes_needed {
        bytes_for_encryption.extend(u32_to_4_u8(current_state).iter());
    }

    bytes_for_encryption.truncate(bytes_needed);
    bytes_for_encryption
}

enum Mode {
    Encrypt,
    Decrypt,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let mode = if opt.decrypt.is_some() {
        Mode::Decrypt
    } else if opt.encrypt.is_some() {
        Mode::Encrypt
    } else {
        panic!("Must provide one of --encrypt or --decrypt")
    };

    let plaintext = {
        let file_or_string = opt.decrypt.or(opt.encrypt).unwrap();

        if let Ok(mut file) = File::open(&file_or_string) {
            let mut buf = vec![];
            file.read_to_end(&mut buf)?;
            buf
        } else {
            file_or_string.as_bytes().to_owned()
        }
    };

    let pad = create_pad(plaintext.len(), opt.cellular_automata, opt.iterations);

    let ciphertext = plaintext
        .iter()
        .zip(pad.iter())
        .map(|(plaintext, pad)| plaintext.bitxor(pad))
        .collect::<Vec<u8>>();

    match mode {
        Mode::Decrypt => println!("{}", String::from_utf8_lossy(&ciphertext)),
        Mode::Encrypt => {
            let mut out_file = File::create("out.txt")?;
            out_file.write_all(&ciphertext)?;
        }
    }

    Ok(())
}
