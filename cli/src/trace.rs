// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use log::LevelFilter;
use std::io::BufWriter;
use std::io::Write;
use std::str::FromStr;

use anyhow::Result;
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;

#[allow(dyn_drop)]
pub async fn init_logging(
    dir: &str,
    level: &str,
) -> Result<Vec<Box<dyn Drop + Send + Sync + 'static>>> {
    let mut guards: Vec<Box<dyn Drop + Send + Sync + 'static>> = Vec::new();
    let mut logger = fern::Dispatch::new();

    let rolling = RollingFileAppender::new(Rotation::DAILY, dir, "bendsql");
    let (non_blocking, flush_guard) = tracing_appender::non_blocking(rolling);
    let buffered_non_blocking = BufWriter::with_capacity(64 * 1024 * 1024, non_blocking);

    guards.push(Box::new(flush_guard));
    logger = logger.chain(
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}] - {} - [{}] {}",
                    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(LevelFilter::from_str(level)?)
            .chain(Box::new(buffered_non_blocking) as Box<dyn Write + Send>),
    );

    if logger.apply().is_err() {
        eprintln!("logger has already been set");
        return Ok(Vec::new());
    }

    Ok(guards)
}
