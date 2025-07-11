use core::str;
use std::clone::Clone;
use std::fmt::Debug;
use std::time::Duration;
use anyhow::{Context,Result,bail};
use futures::future::join_all;
use itertools::Itertools;
use serde::{Serialize,Deserialize};
use serde_json::Value;
use serde_inline_default::serde_inline_default;
use alloy::transports::{RpcError, TransportErrorKind};
use alloy::rpc::client::{ClientBuilder,BatchRequest};
use alloy::transports::http::reqwest::Url;
use duration_str::deserialize_duration;
use super::retry_layer::{RetryLayer,RetryConfig};
use super::concurrency_limit_layer::ConcurrencyLimitLayer;

pub struct RpcCall {
    pub method: String,
    pub params: Vec<Value>
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum RpcResult {
    #[serde(rename = "value")]
    Value(Value),
    #[serde(rename = "error")]
    Error(String)
}

pub type BatchResult = Result<Vec<RpcResult>>;

#[serde_inline_default]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct RpcClientConfig {
    #[serde(alias = "disable-batch")]
    #[serde_inline_default(false)]
    disable_batch: bool,

    #[serde(alias = "max-batch-size")]
    #[serde_inline_default(100)]
    max_batch_size: usize,

    #[serde(alias = "max-concurrent-requests")]
    #[serde_inline_default(5)]
    max_concurrent_requests: usize,

    #[serde(alias = "fail-on-null")]
    #[serde_inline_default(false)]
    fail_on_null: bool,

    #[serde(alias = "fail-on-error")]
    #[serde_inline_default(false)]
    fail_on_error: bool,

    #[serde(alias = "retryable-status-codes")]
    #[serde_inline_default(vec![429, 502, 503])]
    retryable_status_codes: Vec<u16>,

    #[serde(alias = "retry-initial-interval")]
    #[serde(deserialize_with = "deserialize_duration")]
    #[serde_inline_default(Duration::from_millis(100))]
    retry_initial_interval: Duration,

    #[serde(alias = "retry-randomization-factor")]
    #[serde_inline_default(0.5)]
    retry_randomization_factor: f64,

    #[serde(alias = "multiplier")]
    #[serde_inline_default(2.0)]
    retry_multiplier: f64,

    #[serde(alias = "retry-max-interval")]
    #[serde(deserialize_with = "deserialize_duration")]
    #[serde_inline_default(Duration::from_secs(30))]
    retry_max_interval: Duration,

    #[serde(alias = "retry-max-elapsed-time")]
    #[serde(deserialize_with = "deserialize_duration")]
    #[serde_inline_default(Duration::from_secs(60))]
    retry_max_elapsed_time: Duration,

    #[serde(alias = "retry-max-tries")]
    #[serde_inline_default(10)]
    retry_max_tries: u32
}

pub struct RpcClient {
    client: alloy::rpc::client::RpcClient,
    conf: RpcClientConfig
}

impl RpcClient {
    pub fn new(endpoint: &str) -> Result<Self> {
        let u = Url::parse(endpoint)?;
        let conf: RpcClientConfig = serde_qs::from_str(u.fragment().unwrap_or_default())?;
        let client = ClientBuilder::default()
            .layer(RetryLayer::new(RetryConfig{
                retryable_status_codes: conf.retryable_status_codes.clone(),
                initial_interval: conf.retry_initial_interval,
                randomization_factor: conf.retry_randomization_factor,
                multiplier: conf.retry_multiplier,
                max_interval: conf.retry_max_interval,
                max_elapsed_time: conf.retry_max_elapsed_time,
                max_tries: conf.retry_max_tries
            }))
            .layer(ConcurrencyLimitLayer::new(conf.max_concurrent_requests))
            .http(u);

        Ok(RpcClient{client, conf})
    }

    pub async fn calls<I>(&self, calls: I) -> BatchResult
    where
        I: IntoIterator<Item = RpcCall> + Send,
    {
        self.batch_call(calls).await
    }

    pub async fn batch_call<I>(&self, calls: I) -> BatchResult 
    where
        I: IntoIterator<Item = RpcCall> + Send,
    {      
        let mut call_futs = Vec::new();

        let batch_call_futs = calls
            .into_iter()
            .chunks(self.conf.max_batch_size)
            .into_iter()
            .map(|calls| {
                let mut rpc_batch: BatchRequest<'_> = self.client.new_batch();

                for call in calls {
                    call_futs.push(rpc_batch.add_call(call.method, &call.params)?);
                }

                Ok(rpc_batch.send())
            })
            .collect::<Result<Vec<_>>>()?;

        join_all(batch_call_futs).await
            .into_iter()
            .try_for_each(|x| x.context("HTTP call error"))?;
        
        join_all(call_futs).await
            .into_iter()
            .map(|res| self.process_rpc_result(res))
            .collect::<BatchResult>()
    }

    pub async fn multi_call<I>(&self, calls: I) -> BatchResult
    where
        I: IntoIterator<Item = RpcCall> + Send,
    {
        let call_futs = calls
            .into_iter()
            .map(|call| self.client.request(call.method, call.params));

            join_all(call_futs).await
                .into_iter()
                .map(|res| self.process_rpc_result(res))
                .collect::<BatchResult>()
    }

    fn process_rpc_result(&self, res: Result<Value, RpcError<TransportErrorKind>>) -> Result<RpcResult>{
        match res {
            Ok(Value::Null) if self.conf.fail_on_null => bail!("null value"),
            Ok(v) => Ok(RpcResult::Value(v)),
            Err(RpcError::ErrorResp(e)) if !self.conf.fail_on_error => Ok(RpcResult::Error(e.to_string())),
            Err(e) => bail!(e)
        }
    }
}