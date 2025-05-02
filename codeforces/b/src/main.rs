use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::io::{BufWriter, StdoutLock, Write};
use std::str::FromStr;
use std::{cmp, io, usize};

// https://codeforces.com/blog/entry/96067
#[allow(dead_code)]
fn read_line() -> String {
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("failed to read line");

    buffer
}

#[allow(dead_code)]
fn read_vec<T: FromStr>() -> Result<Vec<T>, T::Err> {
    read_line()
        .split_whitespace()
        .map(|x| x.parse::<T>())
        .collect()
}

#[allow(dead_code)]
fn read<T: FromStr>() -> Result<T, T::Err> {
    read_line().trim().parse::<T>()
}

fn ans(out: &mut BufWriter<StdoutLock>, chars: &Vec<char>) {
    for i in chars {
        write!(out, "{}", i).unwrap();
    }
    writeln!(out).unwrap();
}

fn solve(out: &mut BufWriter<StdoutLock>, s: String) -> Option<()> {
    let n = s.len();
    let bits: Vec<char> = s.chars().collect();

    let start = bits[0];

    if bits.iter().all(|&x| x == start) {
        ans(out, &bits);
        return Some(());
    }

    let mut res = vec![];

    for i in 0..n - 1 {
        if bits[i] == bits[i + 1] && bits[i] == '0' {
            res.push('0');
            res.push('1');
        } else if bits[i] == bits[i + 1] && bits[i] == '1' {
            res.push('1');
            res.push('0');
        } else {
            res.push(bits[i]);
        }
    }

    res.push(bits[n - 1]);

    ans(out, &res);

    Some(())
}

fn main() {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let n = read::<usize>().unwrap();

    for _ in 0..n {
        let s = read::<String>().unwrap();

        solve(&mut out, s).unwrap();
    }
}
