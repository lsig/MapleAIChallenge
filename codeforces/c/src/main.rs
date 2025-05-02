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

fn solve(
    out: &mut BufWriter<StdoutLock>,
    k: usize,
    a_n: usize,
    a: Vec<usize>,
    b: Vec<usize>,
) -> Option<()> {
    let mut map: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    let mut count = 0;

    for i in 0..k {
        let boy = a[i];
        let girl = b[i] + a_n;

        map.entry(boy)
            .and_modify(|v| v.push((boy, girl)))
            .or_insert(vec![(boy, girl)]);

        map.entry(girl)
            .and_modify(|v| v.push((boy, girl)))
            .or_insert(vec![(boy, girl)]);
    }

    for i in 0..k {
        let boy = a[i];
        let girl = b[i] + a_n;

        let pairs1 = map.get(&boy)?;
        let pairs2 = map.get(&girl)?;
        let union = pairs1.len() + pairs2.len() - 1;

        count += k - union;
    }

    writeln!(out, "{:?}", count / 2).unwrap();

    Some(())
}

fn main() {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let n = read::<usize>().unwrap();

    for t in 0..n {
        let input = read_vec::<usize>().unwrap();
        let a_n = input[0];
        let k = input[2];

        let a = read_vec::<usize>().unwrap();
        let b = read_vec::<usize>().unwrap();

        // if t == 132 {
        //     writeln!(out, "{:?}{:?}{:?}", input, a, b).unwrap();
        // }

        solve(&mut out, k, a_n, a, b).unwrap();
    }
}
