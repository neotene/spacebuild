pub mod trace {
    use colored::Colorize;
    use regex::Regex;
    use std::io::Write;
    use std::thread;
    use tokio::time::Instant;

    const THREAD_ID_REGEX_STR: &str = "ThreadId\\(([[:digit:]]+)\\)";

    pub fn init(maybe_filter: Option<String>) {
        let launch_time = Instant::now();
        let target_regex_str = if let Some(filter) = maybe_filter {
            filter
        } else {
            format!("(.*)")
        };

        let mut binding = env_logger::builder();
        let builder = binding.format(move |buf, record| {
            let target_str = record.target();
            let regex = Regex::new(target_regex_str.as_str()).unwrap();
            let mut results = vec![];
            for (_, [target]) in regex.captures_iter(target_str).map(|c| c.extract()) {
                results.push(target);
            }

            if results.len() != 1 {
                return write!(buf, "<target>");
            }

            let target_str = results.last().unwrap();

            let thread_id_str = format!("{:?}", thread::current().id());
            let regex = Regex::new(THREAD_ID_REGEX_STR).unwrap();
            let mut results = vec![];

            for (_, [id]) in regex
                .captures_iter(thread_id_str.as_str())
                .map(|c| c.extract())
            {
                results.push(id);
            }
            assert_eq!(1, results.len());
            let thread_id_str = results.last().unwrap();

            let now_time = Instant::now();
            let elapsed = now_time - launch_time;
            let elapsed = elapsed.as_millis() as f32 / 1000.;

            let args_str = format!("{}", record.args());

            writeln!(
                buf,
                "{:<8}{:<4}{:<15}{}",
                elapsed.to_string().blue().bold(),
                thread_id_str.green().bold(),
                target_str.red().bold(),
                args_str,
            )
        });
        builder.init();
    }
}
