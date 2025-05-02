use std::cmp::max;
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

fn solve(out: &mut BufWriter<StdoutLock>, n: usize, mut x: u128) -> Option<()> {
    let mut count = 0;

    while x.to_string().len() < n {
        let chars: HashSet<u32> = x
            .to_string()
            .chars()
            .map(|v| v.to_digit(10).unwrap())
            .collect();

        if chars.iter().all(|&v| v < 2) {
            writeln!(out, "{}", -1).unwrap();
            return Some(());
        }

        let mut next_i = 0;
        let mut max = 0;

        for &i in &chars {
            if i < 2 {
                continue;
            }

            let tmp = x * i as u128;

            let tmp_ch: Vec<u32> = tmp
                .to_string()
                .chars()
                .map(|v| v.to_digit(10).unwrap())
                .collect();

            let tmp_max = tmp_ch.iter().max()? * i;

            if tmp_max > max {
                max = tmp_max;
                next_i = i;
            }
        }

        x = x * next_i as u128;
        count += 1;
    }

    writeln!(out, "{}", count).unwrap();

    Some(())
}

fn main() {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let input = read_vec::<usize>().unwrap();
    let n = input[0];
    let x = input[1] as u128;

    solve(&mut out, n, x).unwrap();
}
