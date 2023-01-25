// Copyright (C) 2022 Electronic Arts, Inc. All rights reserved.

//! Generic functionality for all output modules.
//!
//! A job may consist of one or more commands that need to be executed for successful completion of the job.
use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::{sync::mpsc, time::Instant};

use crate::{
    helm::{HelmResult, Installation},
    layer::LogEntry,
    Task,
};

pub mod slack;
pub mod text;
pub mod tui;

/// A message to the output module.
#[derive(Clone, Debug)]
pub enum Message {
    /// An job is to be skipped.
    SkippedJob(Installation),
    /// A new job that is not going to be skipped.
    NewJob(Installation),
    /// The version data for a job - only if outdated report requested.
    InstallationVersion(Installation, String, String),
    /// The result of running a single command for a job.
    InstallationResult(HelmResult),
    /// Notification that we started a job.
    StartedJob(Installation, Instant),
    /// Notification that we finished a job.
    FinishedJob(Installation, Result<(), String>, Duration),

    /// A Log entry was logged.
    Log(LogEntry),

    /// This gets sent at very start.
    Start(Task, Instant),
    /// This gets sent when all jobs declared finished, but UI should wait for socket to close before ending.
    FinishedAll(Result<(), String>, Duration),
}

#[derive(Clone)]
pub struct MultiOutput {
    tx: Vec<Sender>,
}

impl MultiOutput {
    pub fn new(tx: Vec<Sender>) -> Self {
        Self { tx }
    }

    pub async fn send(&self, msg: Message) {
        for tx in &self.tx {
            tx.send(msg.clone()).await.unwrap_or_else(|err| {
                print!("Cannot send message to output pipe: {err}");
            });
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn try_send(&self, msg: Message) {
        for tx in &self.tx {
            tx.try_send(msg.clone()).unwrap_or_else(|err| {
                print!("Cannot send message to output pipe: {err}");
            });
        }
    }
}

pub type Sender = mpsc::Sender<Message>;

/// Every output module should implement this trait.
#[async_trait]
pub trait Output {
    /// Wait for output to finish.
    async fn wait(&mut self) -> Result<()>;
}
