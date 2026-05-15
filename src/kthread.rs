
#![allow(unused_variables, dead_code, non_snake_case, non_camel_case_types)]

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ktf_worker_t {
    pub i: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct kt_for_t {
    pub n_threads: i32,
    pub n: i64,
    pub w: Vec<ktf_worker_t>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ktp_worker_t<T> {
    pub index: i64,
    pub step: i32,
    pub data: Option<T>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ktp_t<T> {
    pub index: i64,
    pub n_workers: i32,
    pub n_steps: i32,
    pub workers: Vec<ktp_worker_t<T>>,
}

/// Original C static function `steal_work` from `minibwa/kthread.c:30`.
pub fn steal_work(t: &mut kt_for_t) -> i64 {
    let mut min_i = -1i32;
    let mut min = i64::MAX;
    for i in 0..t.n_threads as usize {
        if min > t.w[i].i {
            min = t.w[i].i;
            min_i = i as i32;
        }
    }
    let k = t.w[min_i as usize].i;
    t.w[min_i as usize].i += t.n_threads as i64;
    if k >= t.n {
        -1
    } else {
        k
    }
}

/// Original C static function `ktf_worker` from `minibwa/kthread.c:40`.
pub fn ktf_worker<F>(t: &mut kt_for_t, worker_id: i32, func: &mut F)
where
    F: FnMut(i64, i32),
{
    loop {
        let i = t.w[worker_id as usize].i;
        t.w[worker_id as usize].i += t.n_threads as i64;
        if i >= t.n {
            break;
        }
        func(i, worker_id);
    }
    loop {
        let i = steal_work(t);
        if i < 0 {
            break;
        }
        func(i, worker_id);
    }
}

/// Original C global function `kt_for` from `minibwa/kthread.c:54`.
pub fn kt_for<F>(n_threads: i32, mut func: F, n: i64)
where
    F: FnMut(i64, i32),
{
    if n_threads > 1 {
        let mut t = kt_for_t {
            n_threads,
            n,
            w: (0..n_threads)
                .map(|i| ktf_worker_t { i: i as i64 })
                .collect(),
        };
        for i in 0..n_threads {
            ktf_worker(&mut t, i, &mut func);
        }
    } else {
        for j in 0..n {
            func(j, 0);
        }
    }
}

/// Original C static function `ktp_worker` from `minibwa/kthread.c:97`.
pub fn ktp_worker<S, T, F>(p: &mut ktp_t<T>, worker_id: i32, shared: &mut S, func: &mut F)
where
    F: FnMut(&mut S, i32, Option<T>) -> Option<T>,
{
    let wid = worker_id as usize;
    while p.workers[wid].step < p.n_steps {
        let step = p.workers[wid].step;
        let input = if step != 0 {
            p.workers[wid].data.take()
        } else {
            None
        };
        p.workers[wid].data = func(shared, step, input);
        if step == p.n_steps - 1 || p.workers[wid].data.is_some() {
            p.workers[wid].step = (step + 1) % p.n_steps;
        } else {
            p.workers[wid].step = p.n_steps;
        }
        if p.workers[wid].step == 0 {
            p.workers[wid].index = p.index;
            p.index += 1;
        }
    }
}

/// Original C global function `kt_pipeline` from `minibwa/kthread.c:130`.
pub fn kt_pipeline<S, T, F>(n_threads: i32, mut func: F, shared_data: &mut S, n_steps: i32)
where
    F: FnMut(&mut S, i32, Option<T>) -> Option<T>,
{
    let n_threads = n_threads.max(1);
    let mut aux = ktp_t {
        index: 0,
        n_workers: n_threads,
        n_steps,
        workers: Vec::new(),
    };
    for _ in 0..n_threads {
        aux.workers.push(ktp_worker_t {
            index: aux.index,
            step: 0,
            data: None,
        });
        aux.index += 1;
    }
    for i in 0..n_threads {
        ktp_worker(&mut aux, i, shared_data, &mut func);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steal_work_takes_lowest_pending_worker_index() {
        let mut t = kt_for_t {
            n_threads: 3,
            n: 10,
            w: vec![
                ktf_worker_t { i: 8 },
                ktf_worker_t { i: 4 },
                ktf_worker_t { i: 6 },
            ],
        };
        assert_eq!(steal_work(&mut t), 4);
        assert_eq!(t.w[1].i, 7);
    }

    #[test]
    fn kt_for_visits_each_index_once() {
        let mut seen = Vec::new();
        kt_for(4, |i, tid| seen.push((i, tid)), 17);
        seen.sort_unstable_by_key(|x| x.0);
        assert_eq!(
            seen.iter().map(|x| x.0).collect::<Vec<_>>(),
            (0..17).collect::<Vec<_>>()
        );
        assert!(seen.iter().all(|x| x.1 >= 0 && x.1 < 4));
    }

    #[test]
    fn kt_pipeline_runs_items_through_all_steps_until_source_empty() {
        #[derive(Default)]
        struct Shared {
            next: i32,
            out: Vec<String>,
        }
        let mut shared = Shared::default();
        kt_pipeline(
            2,
            |s: &mut Shared, step, input: Option<i32>| -> Option<i32> {
                if step == 0 {
                    if s.next < 4 {
                        let v = s.next;
                        s.next += 1;
                        Some(v)
                    } else {
                        None
                    }
                } else if step == 1 {
                    input.map(|v| v * 10)
                } else {
                    let v = input.unwrap();
                    s.out.push(format!("{v}"));
                    Some(v)
                }
            },
            &mut shared,
            3,
        );
        assert_eq!(shared.out, ["0", "10", "20", "30"]);
    }
}
