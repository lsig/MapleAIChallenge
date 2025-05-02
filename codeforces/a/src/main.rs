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

fn solve(out: &mut BufWriter<StdoutLock>, s: String) -> Option<()> {
    let n = s.len();

    if n % 2 != 0 {
        writeln!(out, "NO").unwrap();
        return Some(());
    }

    let (s1, s2) = s.split_at(n / 2);

    if s1 == s2 {
        writeln!(out, "YES").unwrap();
    } else {
        writeln!(out, "NO").unwrap();
    }

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
