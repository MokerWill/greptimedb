// Copyright 2023 Greptime Team
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

use api::v1::value::ValueData;
use api::v1::{Rows, Value};
use common_telemetry::tracing::info;
use greptime_proto::v1::value::ValueData::{
    BinaryValue, BoolValue, F64Value, StringValue, TimestampNanosecondValue, TimestampSecondValue,
    U32Value, U64Value, U8Value,
};
use greptime_proto::v1::Value as GreptimeValue;
use pipeline::{parse, setup_pipeline, Content, Pipeline, PipelineContext};

#[test]
fn test_complex_data() {
    let input_value_str = r#"
      {
        "version": 1,
        "streamId": "12345",
        "cp": "123456",
        "reqId": "1239f220",
        "reqTimeSec": "1573840000",
        "bytes": "4995",
        "cliIP": "128.147.28.68",
        "statusCode": "206",
        "proto": "HTTPS",
        "reqHost": "test.hostname.net",
        "reqMethod": "GET",
        "reqPath": "/path1/path2/file.ext",
        "reqPort": "443",
        "rspContentLen": "5000",
        "rspContentType": "text/html",
        "UA": "Mozilla%2F5.0+%28Macintosh%3B+Intel+Mac+OS+X+10_14_3%29",
        "tlsOverheadTimeMSec": "0",
        "tlsVersion": "TLSv1",
        "objSize": "484",
        "uncompressedSize": "484",
        "overheadBytes": "232",
        "totalBytes": "0",
        "queryStr": "cmcd=//1.0@V/bl=21600,br=1426,cid=%22akam-email%22,d=6006,mtp=11100,ot=m,sf=h,sid=%229f36f5c9-d6a2-497b-8c73-4b8f694eab749f36f5c9-d6a2-497b-8c73%22,tb=1426,dl=18500,nor=%22../300kbps/track.m4v%22,nrr=%2212323-48763%22,su,bs,rtp=12000,pr=1.08,sf=d,st=v%22",
        "breadcrumbs": "//BC/%5Ba=23.33.41.20,c=g,k=0,l=1%5D",
        "accLang": "en-US",
        "cookie": "cookie-content",
        "range": "37334-42356",
        "referer": "https%3A%2F%2Ftest.referrer.net%2Fen-US%2Fdocs%2FWeb%2Ftest",
        "xForwardedFor": "8.47.28.38",
        "maxAgeSec": "3600",
        "reqEndTimeMSec": "3",
        "errorCode": "ERR_ACCESS_DENIED|fwd_acl",
        "turnAroundTimeMSec": "11",
        "transferTimeMSec": "125",
        "dnsLookupTimeMSec": "50",
        "lastByte": "1",
        "edgeIP": "23.50.51.173",
        "country": "IN",
        "state": "Virginia",
        "city": "HERNDON",
        "serverCountry": "SG",
        "billingRegion": "8",
        "cacheStatus": "1",
        "securityRules": "ULnR_28976|3900000:3900001:3900005:3900006:BOT-ANOMALY-HEADER|",
        "ewUsageInfo": "//4380/4.0/1/-/0/4/#1,2\\//4380/4.0/4/-/0/4/#0,0\\//4380/4.0/5/-/1/1/#0,0",
        "ewExecutionInfo": "c:4380:7:161:162:161:n:::12473:200|C:4380:3:0:4:0:n:::6967:200|R:4380:20:99:99:1:n:::35982:200",
        "customField": "any-custom-value"
      }
"#;
    let input_value = serde_json::from_str::<serde_json::Value>(input_value_str)
        .expect("failed to parse input value");

    let pipeline_yaml = r#"
---
description: Pipeline for Demo Log

processors:
  - urlencoding:
      fields:
        - breadcrumbs
        - UA
        - referer
        - queryStr
      method: decode
      ignore_missing: true
  - epoch:
      field: reqTimeSec
      resolution: second
      ignore_missing: true
  - regex:
      field: breadcrumbs
      patterns:
        - "(?<parent>\\[[^\\[]*c=c[^\\]]*\\])"
        - "(?<edge>\\[[^\\[]*c=g[^\\]]*\\])"
        - "(?<origin>\\[[^\\[]*c=o[^\\]]*\\])"
        - "(?<peer>\\[[^\\[]*c=p[^\\]]*\\])"
        - "(?<cloud_wrapper>\\[[^\\[]*c=w[^\\]]*\\])"
      ignore_missing: true
  - regex:
      fields:
        - breadcrumbs_parent
        - breadcrumbs_edge
        - breadcrumbs_origin
        - breadcrumbs_peer
        - breadcrumbs_cloud_wrapper
      ignore_missing: true
      patterns:
        - "a=(?<ip>[^,\\]]+)"
        - "b=(?<request_id>[^,\\]]+)"
        - "k=(?<request_end_time>[^,\\]]+)"
        - "l=(?<turn_around_time>[^,\\]]+)"
        - "m=(?<dns_lookup_time>[^,\\]]+)"
        - "n=(?<geo>[^,\\]]+)"
        - "o=(?<asn>[^,\\]]+)"
  - regex:
      field: queryStr, cmcd
      patterns:
        - "(?i)CMCD=//(?<version>[\\d\\.]+)@V/(?<data>.+$)"
      ignore_missing: true
  - cmcd:
      field: cmcd_data, cmcd
      ignore_missing: true

transform:
  - fields:
      - breadcrumbs
      - referer
      - queryStr, query_str
      - customField, custom_field
      - reqId, req_id
      - city
      - state
      - country
      - securityRules, security_rules
      - ewUsageInfo, ew_usage_info
      - ewExecutionInfo, ew_execution_info
      - errorCode, error_code
      - xForwardedFor, x_forwarded_for
      - range
      - accLang, acc_lang
      - reqMethod, req_method
      - reqHost, req_host
      - proto
      - cliIP, cli_ip
      - rspContentType, rsp_content_type
      - tlsVersion, tls_version
    type: string
  - fields:
      - version
      - cacheStatus, cache_status
      - lastByte, last_byte
    type: uint8
  - fields:
      - streamId, stream_id
      - billingRegion, billing_region
      - dnsLookupTimeMSec, dns_lookup_time_msec
      - transferTimeMSec, transfer_time_msec
      - turnAroundTimeMSec, turn_around_time_msec
      - reqEndTimeMSec, req_end_time_msec
      - maxAgeSec, max_age_sec
      - reqPort, req_port
      - statusCode, status_code
      - cp
      - tlsOverheadTimeMSec, tls_overhead_time_msec
    type: uint32
  - fields:
      - bytes
      - rspContentLen, rsp_content_len
      - objSize, obj_size
      - uncompressedSize, uncompressed_size
      - overheadBytes, overhead_bytes
      - totalBytes, total_bytes
    type: uint64
  - fields:
      - UA, user_agent
      - cookie
      - reqPath, req_path
    type: string
    # index: fulltext
  - field: reqTimeSec, req_time_sec
    # epoch time is special, the resolution MUST BE specified
    type: epoch, second
    index: timestamp

  # the following is from cmcd
  - fields:
      - cmcd_version
      - cmcd_cid, cmcd_content_id
      - cmcd_nor, cmcd_next_object_requests
      - cmcd_nrr, cmcd_next_range_request
      - cmcd_ot, cmcd_object_type
      - cmcd_sf, cmcd_streaming_format
      - cmcd_sid, cmcd_session_id
      - cmcd_st, cmcd_stream_type
      - cmcd_v
    type: string
  - fields:
      - cmcd_br, cmcd_encoded_bitrate
      - cmcd_bl, cmcd_buffer_length
      - cmcd_d, cmcd_object_duration
      - cmcd_dl, cmcd_deadline
      - cmcd_mtp, cmcd_measured_throughput
      - cmcd_rtp, cmcd_requested_max_throughput
      - cmcd_tb, cmcd_top_bitrate
    type: uint64
  - fields:
      - cmcd_pr, cmcd_playback_rate
    type: float64
  - fields:
      - cmcd_bs, cmcd_buffer_starvation
      - cmcd_su, cmcd_startup
    type: boolean

  # the following is from breadcrumbs
  - fields:
      - breadcrumbs_parent_ip
      - breadcrumbs_parent_request_id
      - breadcrumbs_parent_geo
      - breadcrumbs_edge_ip
      - breadcrumbs_edge_request_id
      - breadcrumbs_edge_geo
      - breadcrumbs_origin_ip
      - breadcrumbs_origin_request_id
      - breadcrumbs_origin_geo
      - breadcrumbs_peer_ip
      - breadcrumbs_peer_request_id
      - breadcrumbs_peer_geo
      - breadcrumbs_cloud_wrapper_ip
      - breadcrumbs_cloud_wrapper_request_id
      - breadcrumbs_cloud_wrapper_geo
    type: string
  - fields:
      - breadcrumbs_parent_request_end_time
      - breadcrumbs_parent_turn_around_time
      - breadcrumbs_parent_dns_lookup_time
      - breadcrumbs_parent_asn
      - breadcrumbs_edge_request_end_time
      - breadcrumbs_edge_turn_around_time
      - breadcrumbs_edge_dns_lookup_time
      - breadcrumbs_edge_asn
      - breadcrumbs_origin_request_end_time
      - breadcrumbs_origin_turn_around_time
      - breadcrumbs_origin_dns_lookup_time
      - breadcrumbs_origin_asn
      - breadcrumbs_peer_request_end_time
      - breadcrumbs_peer_turn_around_time
      - breadcrumbs_peer_dns_lookup_time
      - breadcrumbs_peer_asn
      - breadcrumbs_cloud_wrapper_request_end_time
      - breadcrumbs_cloud_wrapper_turn_around_time
      - breadcrumbs_cloud_wrapper_dns_lookup_time
      - breadcrumbs_cloud_wrapper_asn
    type: uint32
"#;

    let expected_values = vec![
        (
            "breadcrumbs",
            Some(StringValue("//BC/[a=23.33.41.20,c=g,k=0,l=1]".into())),
        ),
        (
            "referer",
            Some(StringValue(
                "https://test.referrer.net/en-US/docs/Web/test".into(),
            )),
        ),
        (
            "query_str",
            Some(StringValue("cmcd=//1.0@V/bl=21600,br=1426,cid=\"akam-email\",d=6006,mtp=11100,ot=m,sf=h,sid=\"9f36f5c9-d6a2-497b-8c73-4b8f694eab749f36f5c9-d6a2-497b-8c73\",tb=1426,dl=18500,nor=\"../300kbps/track.m4v\",nrr=\"12323-48763\",su,bs,rtp=12000,pr=1.08,sf=d,st=v\"".into())),
        ),
        ("custom_field", Some(StringValue("any-custom-value".into()))),
        ("req_id", Some(StringValue("1239f220".into()))),
        ("city", Some(StringValue("HERNDON".into()))),
        ("state", Some(StringValue("Virginia".into()))),
        ("country", Some(StringValue("IN".into()))),
        (
            "security_rules",
            Some(StringValue(
                "ULnR_28976|3900000:3900001:3900005:3900006:BOT-ANOMALY-HEADER|".into(),
            )),
        ),
        (
            "ew_usage_info",
            Some(StringValue(
                "//4380/4.0/1/-/0/4/#1,2\\//4380/4.0/4/-/0/4/#0,0\\//4380/4.0/5/-/1/1/#0,0".into(),
            )),
        ),
        (
            "ew_execution_info",
            Some(StringValue("c:4380:7:161:162:161:n:::12473:200|C:4380:3:0:4:0:n:::6967:200|R:4380:20:99:99:1:n:::35982:200".into()))),
        (
            "error_code",
            Some(StringValue("ERR_ACCESS_DENIED|fwd_acl".into())),
        ),
        ("x_forwarded_for", Some(StringValue("8.47.28.38".into()))),
        ("range", Some(StringValue("37334-42356".into()))),
        ("acc_lang", Some(StringValue("en-US".into()))),
        ("req_method", Some(StringValue("GET".into()))),
        ("req_host", Some(StringValue("test.hostname.net".into()))),
        ("proto", Some(StringValue("HTTPS".into()))),
        ("cli_ip", Some(StringValue("128.147.28.68".into()))),
        ("rsp_content_type", Some(StringValue("text/html".into()))),
        ("tls_version", Some(StringValue("TLSv1".into()))),
        ("version", Some(U8Value(1))),
        ("cache_status", Some(U8Value(1))),
        ("last_byte", Some(U8Value(1))),
        ("stream_id", Some(U32Value(12345))),
        ("billing_region", Some(U32Value(8))),
        ("dns_lookup_time_msec", Some(U32Value(50))),
        ("transfer_time_msec", Some(U32Value(125))),
        ("turn_around_time_msec", Some(U32Value(11))),
        ("req_end_time_msec", Some(U32Value(3))),
        ("max_age_sec", Some(U32Value(3600))),
        ("req_port", Some(U32Value(443))),
        ("status_code", Some(U32Value(206))),
        ("cp", Some(U32Value(123456))),
        ("tls_overhead_time_msec", Some(U32Value(0))),
        ("bytes", Some(U64Value(4995))),
        ("rsp_content_len", Some(U64Value(5000))),
        ("obj_size", Some(U64Value(484))),
        ("uncompressed_size", Some(U64Value(484))),
        ("overhead_bytes", Some(U64Value(232))),
        ("total_bytes", Some(U64Value(0))),
        (
            "user_agent",
            Some(StringValue(
                "Mozilla/5.0+(Macintosh;+Intel+Mac+OS+X+10_14_3)".into(),
            )),
        ),
        ("cookie", Some(StringValue("cookie-content".into()))),
        (
            "req_path",
            Some(StringValue("/path1/path2/file.ext".into())),
        ),
        ("req_time_sec", Some(TimestampSecondValue(1573840000))),
        ("cmcd_version", Some(StringValue("1.0".into()))),
        (
            "cmcd_content_id",
            Some(StringValue("\"akam-email\"".into())),
        ),
        (
            "cmcd_next_object_requests",
            Some(StringValue("\"../300kbps/track.m4v\"".into())),
        ),
        (
            "cmcd_next_range_request",
            Some(StringValue("\"12323-48763\"".into())),
        ),
        ("cmcd_object_type", Some(StringValue("m".into()))),
        ("cmcd_streaming_format", Some(StringValue("d".into()))),
        (
            "cmcd_session_id",
            Some(StringValue(
                "\"9f36f5c9-d6a2-497b-8c73-4b8f694eab749f36f5c9-d6a2-497b-8c73\"".into(),
            )),
        ),
        ("cmcd_stream_type", Some(StringValue("v\"".into()))),
        ("cmcd_v", None),
        ("cmcd_encoded_bitrate", Some(U64Value(1426))),
        ("cmcd_buffer_length", Some(U64Value(21600))),
        ("cmcd_object_duration", Some(U64Value(6006))),
        ("cmcd_deadline", Some(U64Value(18500))),
        ("cmcd_measured_throughput", Some(U64Value(11100))),
        ("cmcd_requested_max_throughput", Some(U64Value(12000))),
        ("cmcd_top_bitrate", Some(U64Value(1426))),
        ("cmcd_playback_rate", Some(F64Value(1.08))),
        ("cmcd_buffer_starvation", Some(BoolValue(true))),
        ("cmcd_startup", Some(BoolValue(true))),
        ("breadcrumbs_parent_ip", None),
        ("breadcrumbs_parent_request_id", None),
        ("breadcrumbs_parent_geo", None),
        (
            "breadcrumbs_edge_ip",
            Some(StringValue("23.33.41.20".into())),
        ),
        ("breadcrumbs_edge_request_id", None),
        ("breadcrumbs_edge_geo", None),
        ("breadcrumbs_origin_ip", None),
        ("breadcrumbs_origin_request_id", None),
        ("breadcrumbs_origin_geo", None),
        ("breadcrumbs_peer_ip", None),
        ("breadcrumbs_peer_request_id", None),
        ("breadcrumbs_peer_geo", None),
        ("breadcrumbs_cloud_wrapper_ip", None),
        ("breadcrumbs_cloud_wrapper_request_id", None),
        ("breadcrumbs_cloud_wrapper_geo", None),
        ("breadcrumbs_parent_request_end_time", None),
        ("breadcrumbs_parent_turn_around_time", None),
        ("breadcrumbs_parent_dns_lookup_time", None),
        ("breadcrumbs_parent_asn", None),
        ("breadcrumbs_edge_request_end_time", Some(U32Value(0))),
        ("breadcrumbs_edge_turn_around_time", Some(U32Value(1))),
        ("breadcrumbs_edge_dns_lookup_time", None),
        ("breadcrumbs_edge_asn", None),
        ("breadcrumbs_origin_request_end_time", None),
        ("breadcrumbs_origin_turn_around_time", None),
        ("breadcrumbs_origin_dns_lookup_time", None),
        ("breadcrumbs_origin_asn", None),
        ("breadcrumbs_peer_request_end_time", None),
        ("breadcrumbs_peer_turn_around_time", None),
        ("breadcrumbs_peer_dns_lookup_time", None),
        ("breadcrumbs_peer_asn", None),
        ("breadcrumbs_cloud_wrapper_request_end_time", None),
        ("breadcrumbs_cloud_wrapper_turn_around_time", None),
        ("breadcrumbs_cloud_wrapper_dns_lookup_time", None),
        ("breadcrumbs_cloud_wrapper_asn", None),
    ]
    .into_iter()
    .map(|(_, d)| GreptimeValue { value_data: d })
    .collect::<Vec<GreptimeValue>>();

    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).expect("failed to parse pipeline");
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );
    let stats = input_value.into();

    let row = pipeline
        .exec_mut(stats, &pipeline_ctx, &mut schema_info)
        .expect("failed to exec pipeline")
        .into_transformed()
        .expect("expect transformed result ");

    let output = Rows {
        schema: pipeline.schemas().unwrap().clone(),
        rows: vec![row.0],
    };

    assert_eq!(output.rows.len(), 1);
    let values = output.rows.first().unwrap().values.clone();
    assert_eq!(expected_values, values);

    for s in output.schema.iter() {
        info!(
            "{}({}): {}",
            s.column_name,
            s.datatype().as_str_name(),
            s.semantic_type().as_str_name()
        );
    }
    info!("\n");

    let get_schema_name = |ss: &Vec<greptime_proto::v1::ColumnSchema>, i: usize| {
        let s = ss.get(i).unwrap();
        s.column_name.clone()
    };

    for row in output.rows.iter() {
        let values = &row.values;
        for i in 0..values.len() {
            let val = values.get(i).unwrap();
            info!(
                "{}: {:?}, ",
                get_schema_name(&output.schema, i),
                val.value_data
            );
        }
        info!("\n");
    }
}

#[test]
fn test_json_type() {
    let input_value_str = r#"
{
    "product_object": {"hello":"world"},
    "product_array": ["hello", "world"]
}
"#;
    let input_value = serde_json::from_str::<serde_json::Value>(input_value_str).unwrap();

    let pipeline_yaml = r#"
processors:

transform:
    - fields:
        - product_object
        - product_array
      type: json
"#;

    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).unwrap();
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );

    let status = input_value.into();
    let row = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap()
        .into_transformed()
        .expect("expect transformed result ");
    let r = row
        .0
        .values
        .into_iter()
        .map(|v| v.value_data.unwrap())
        .collect::<Vec<_>>();

    let product_object = r[0].clone();
    let product_array = r[1].clone();
    match product_object {
        ValueData::BinaryValue(data) => {
            let jsonb = jsonb::from_slice(&data).unwrap().to_string();
            assert_eq!(r#"{"hello":"world"}"#, jsonb);
        }
        _ => panic!("unexpected value"),
    }

    match product_array {
        ValueData::BinaryValue(data) => {
            let jsonb = jsonb::from_slice(&data).unwrap().to_string();
            assert_eq!(r#"["hello","world"]"#, jsonb);
        }
        _ => panic!("unexpected value"),
    }
}

#[test]
fn test_json_path() {
    let input_value_str = r#"
{
  "product_object": {
    "hello": "world"
  },
  "product_array": [
    "hello",
    "world"
  ],
  "complex_object": {
    "shop": {
      "orders": [
        {
          "id": 1,
          "active": true
        },
        {
          "id": 2
        },
        {
          "id": 3
        },
        {
          "id": 4,
          "active": true
        }
      ]
    }
  }
}"#;
    let input_value = serde_json::from_str::<serde_json::Value>(input_value_str).unwrap();

    let pipeline_yaml = r#"
processors:
    - json_path:
        fields:
            - product_object, object_target
        json_path: "$.hello"
        result_index: 0
    - json_path:
        fields:
            - product_array, array_target
        json_path: "$.[1]"
        result_index: 0
    - json_path:
        fields:
            - complex_object, complex_target1
        json_path: "$.shop.orders[?(@.active)].id"
    - json_path:
        fields:
            - complex_target1, complex_target_2
        json_path: "$.[1]"
        result_index: 0
    - json_path:
        fields:
            - complex_object, complex_target_3
        json_path: "$.shop.orders[?(@.active)].id"
        result_index: 1
transform:
    - fields:
        - object_target
        - array_target
      type: string
    - fields:
        - complex_target_3
        - complex_target_2
      type: uint32
    - fields:
        - complex_target1
      type: json
"#;

    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).unwrap();
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );

    let status = input_value.into();
    let row = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap()
        .into_transformed()
        .expect("expect transformed result ");

    let r = row
        .0
        .values
        .into_iter()
        .map(|v| v.value_data.unwrap())
        .collect::<Vec<_>>();

    let object_target = r[0].clone();
    let array_target = r[1].clone();
    let complex_target3 = r[2].clone();
    let complex_target2 = r[3].clone();
    let complex_target1 = r[4].clone();

    assert_eq!(StringValue("world".into()), object_target);
    assert_eq!(StringValue("world".into()), array_target);
    assert_eq!(complex_target3, complex_target2);

    assert_eq!(
        BinaryValue(
            jsonb::Value::Array(vec![jsonb::Value::from(1), jsonb::Value::from(4),]).to_vec()
        ),
        complex_target1
    );
}

#[test]
fn test_simple_data() {
    let input_value_str = r#"
{
    "line": "2024-05-25 20:16:37.217 hello world"
}
"#;
    let input_value = serde_json::from_str::<serde_json::Value>(input_value_str).unwrap();

    let pipeline_yaml = r#"
processors:
  - dissect:
      fields:
        - line
      patterns:
        - "%{+ts} %{+ts} %{content}"
  - date:
      fields:
        - ts
      formats:
        - "%Y-%m-%d %H:%M:%S%.3f"

transform:
  - fields:
      - content
    type: string
  - field: ts
    type: time
    index: timestamp
"#;

    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).unwrap();
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );

    let status = input_value.into();
    let row = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap()
        .into_transformed()
        .expect("expect transformed result ");
    let r = row
        .0
        .values
        .into_iter()
        .map(|v| v.value_data.unwrap())
        .collect::<Vec<_>>();

    let expected = vec![
        StringValue("hello world".into()),
        TimestampNanosecondValue(1716668197217000000),
    ];

    assert_eq!(expected, r);
}

#[test]
fn test_decolorize() {
    let input_value = serde_json::json!({
        "message": "\u{001b}[32mSuccess\u{001b}[0m and \u{001b}[31mError\u{001b}[0m"
    });

    let pipeline_yaml = r#"
processors:
  - decolorize:
      fields:
        - message
transform:
  - fields:
      - message
    type: string
"#;
    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).unwrap();
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );

    let status = input_value.into();
    let row = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap()
        .into_transformed()
        .expect("expect transformed result ");

    let r = row
        .0
        .values
        .into_iter()
        .map(|v| v.value_data.unwrap())
        .collect::<Vec<_>>();

    let expected = StringValue("Success and Error".into());
    assert_eq!(expected, r[0]);
}

#[test]
fn test_digest() {
    let input_value = serde_json::json!({
        "message": "hello world",
        "message_with_ip": "hello 192.168.1.1 world",
        "message_with_uuid": "hello 123e4567-e89b-12d3-a456-426614174000 world",
        "message_with_quote": "hello 'quoted text' world",
        "message_bracket": "hello [bracketed text] world",
        "message_with_foobar": "hello foobar world"
    });

    let pipeline_yaml = r#"
processors:
  - digest:
      fields:
        - message
        - message_with_ip
        - message_with_uuid
        - message_with_quote
        - message_bracket
        - message_with_foobar
      presets:
        - ip
        - uuid
        - bracketed
        - quoted
      regex:
        - foobar
transform:
  - fields:
      - message_with_ip_digest
      - message_with_uuid_digest
      - message_with_quote_digest
      - message_bracket_digest
      - message_with_foobar_digest
    type: string
"#;

    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).unwrap();
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );

    let status = input_value.into();
    let row = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap()
        .into_transformed()
        .expect("expect transformed result ");

    let mut r = row
        .0
        .values
        .into_iter()
        .map(|v| v.value_data.unwrap())
        .collect::<Vec<_>>();
    r.pop(); // remove the timestamp value

    let expected = vec![
        StringValue("hello  world".into()),
        StringValue("hello  world".into()),
        StringValue("hello  world".into()),
        StringValue("hello  world".into()),
        StringValue("hello  world".into()),
    ];

    assert_eq!(expected, r);
}

#[test]
fn test_timestamp_default_now() {
    let input_value = serde_json::json!({"abc": "hello world"});

    let pipeline_yaml = r#"
processors:
transform:
    - field: abc
      type: string
      on_failure: default
"#;

    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).unwrap();
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );

    let status = input_value.into();
    let row = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap()
        .into_transformed()
        .expect("expect transformed result ");

    row.0.values.into_iter().for_each(|v| {
        if let ValueData::TimestampNanosecondValue(v) = v.value_data.unwrap() {
            let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
            assert!(now - v < 1_000_000);
        }
    });
}

#[test]
fn test_dispatch() {
    let input_value_str1 = r#"
{
    "line": "2024-05-25 20:16:37.217 [http] hello world"
}
"#;
    let input_value1 = serde_json::from_str::<serde_json::Value>(input_value_str1).unwrap();
    let input_value_str2 = r#"
{
    "line": "2024-05-25 20:16:37.217 [database] hello world"
}
"#;
    let input_value2 = serde_json::from_str::<serde_json::Value>(input_value_str2).unwrap();

    let pipeline_yaml = r#"
processors:
  - dissect:
      fields:
        - line
      patterns:
        - "%{+ts} %{+ts} [%{logger}] %{content}"
  - date:
      fields:
        - ts
      formats:
        - "%Y-%m-%d %H:%M:%S%.3f"

dispatcher:
  field: logger
  rules:
    - value: http
      table_suffix: http
      pipeline: access_log_pipeline

transform:
  - fields:
      - content
    type: string
  - field: ts
    type: time
    index: timestamp
"#;

    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).unwrap();
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );

    let status = input_value1.into();
    let dispatched_to = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap()
        .into_dispatched()
        .expect("expect dispatched result ");
    assert_eq!(dispatched_to.table_suffix, "http");
    assert_eq!(dispatched_to.pipeline.unwrap(), "access_log_pipeline");

    let status = input_value2.into();
    let row = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap()
        .into_transformed()
        .expect("expect transformed result ");
    let r = row
        .0
        .values
        .into_iter()
        .map(|v| v.value_data.unwrap())
        .collect::<Vec<_>>();

    let expected = vec![
        StringValue("hello world".into()),
        TimestampNanosecondValue(1716668197217000000),
    ];

    assert_eq!(expected, r);
}

#[test]
fn test_table_suffix_template() {
    let input_value = r#"
{
    "line": "2024-05-25 20:16:37.217 [http] hello world"
}
"#;
    let input_value = serde_json::from_str::<serde_json::Value>(input_value).unwrap();

    let pipeline_yaml = r#"
processors:
  - dissect:
      fields:
        - line
      patterns:
        - "%{+ts} %{+ts} [%{logger}] %{content}"
  - date:
      fields:
        - ts
      formats:
        - "%Y-%m-%d %H:%M:%S%.3f"
transform:
  - fields:
      - content
    type: string
  - field: ts
    type: time
    index: timestamp
table_suffix: _${logger}
"#;

    let yaml_content = Content::Yaml(pipeline_yaml);
    let pipeline: Pipeline = parse(&yaml_content).unwrap();
    let (pipeline, mut schema_info, pipeline_def, pipeline_param) = setup_pipeline!(pipeline);
    let pipeline_ctx = PipelineContext::new(
        &pipeline_def,
        &pipeline_param,
        session::context::Channel::Unknown,
    );

    let status = input_value.into();
    let exec_re = pipeline
        .exec_mut(status, &pipeline_ctx, &mut schema_info)
        .unwrap();

    let (row, table_name) = exec_re.into_transformed().unwrap();
    let values = row.values;
    let expected_values = vec![
        Value {
            value_data: Some(ValueData::StringValue("hello world".into())),
        },
        Value {
            value_data: Some(ValueData::TimestampNanosecondValue(1716668197217000000)),
        },
    ];
    assert_eq!(expected_values, values);
    assert_eq!(table_name, Some("_http".to_string()));
}
